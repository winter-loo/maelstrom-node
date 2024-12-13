use crate::messages::*;
use crate::node::Node;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

const SVC: &str = "lin-kv";

// range partition for keys
// root key: ["n0-1", "n0-2", "n1-1", "n1-2"]
// "n0-1": [1 "n0-3", 2 "n1-3" 7 "n2-3"]
// "n0-2": [12 "n0-8" 13 "n1-8" 14 "n2-8"]
// "n1-1": [21 "n0-3", 22 "n1-3" 27 "n2-3"]
const DB_PARTITION_KEY: &str = "ROOT";

struct Root {
    node: Rc<RefCell<Node>>,
    // [(0, "n0-1"), (1, "n0-2"), (2, "n1-1"), (3, "n1-2")]
    // (partition_key, partition_id)
    part_keys: HashMap<usize, String>,
    // (partition_id, (key, chunk_id))
    parts: RefCell<HashMap<String, HashMap<usize, String>>>,
}

impl Root {
    fn load(node: &Rc<RefCell<Node>>) -> Self {
        let res = node.borrow_mut().sync_rpc(
            SVC,
            MessageExtra::KvRead(KvReadExtra {
                key: DB_PARTITION_KEY.into(),
            }),
        );

        let part_keys = match res {
            MessageExtra::KvReadOk(v) => serde_json::from_value(v.value).unwrap(),
            MessageExtra::Error(err) => {
                if err.code == 20 {
                    HashMap::new()
                } else {
                    panic!("wrong response for lin-kv read")
                }
            }
            _ => panic!("wrong response for lin-kv read"),
        };
        eprintln!("root partition keys: {part_keys:#?}");

        Root {
            node: node.clone(),
            part_keys,
            parts: RefCell::new(HashMap::new()),
        }
    }

    fn load_partition(&self, key: &str) -> HashMap<usize, String> {
        let res = self
            .node
            .borrow_mut()
            .sync_rpc(SVC, MessageExtra::KvRead(KvReadExtra { key: key.into() }));

        match res {
            MessageExtra::KvReadOk(v) => serde_json::from_value(v.value).unwrap(),
            MessageExtra::Error(err) => {
                if err.code == 20 {
                    HashMap::new()
                } else {
                    panic!("wrong response for lin-kv read")
                }
            }
            _ => panic!("wrong response for lin-kv read"),
        }
    }

    fn save_partition(&self, key: &str, value: &HashMap<usize, String>) {
        eprintln!("save partiton: {key}, {value:#?}");
        let res = self.node.borrow_mut().sync_rpc(
            SVC,
            MessageExtra::KvWrite(KvWriteExtra {
                key: key.into(),
                value: serde_json::to_value(value.clone()).unwrap(),
            }),
        );
        if !matches!(res, MessageExtra::KvWriteOk) {
            unreachable!("the write error is not expected");
        }
    }

    fn next_part_id(&self, partition_key: &usize) -> String {
        static mut NEXT_ID: usize = 0;
        unsafe {
            NEXT_ID += 1;
        }
        let id = unsafe { NEXT_ID };
        format!("part-{}-{}-{}", partition_key, self.node.borrow().id, id)
    }

    fn part_key(&self, key: &usize) -> usize {
        // this function set the size of a partition.
        key / 20
    }

    /// saved: [(key, chunk_id)]
    fn save(&self, saved: &HashMap<usize, String>) -> bool {
        eprintln!("save {saved:#?}");
        let mut new_part_keys = self.part_keys.clone();
        eprintln!("new part keys: {new_part_keys:#?}");

        // which partition has changed?
        let mut new_parts: HashMap<String, HashMap<usize, String>> = HashMap::new();
        saved.iter().for_each(|(k, v)| {
            let partition_key = self.part_key(k);
            match new_part_keys.get(&partition_key) {
                Some(pid) => {
                    if new_parts.contains_key(pid) {
                        let part = new_parts.get_mut(pid).unwrap();
                        part.insert(*k, v.to_string());
                    } else {
                        let new_pid = self.next_part_id(&partition_key);
                        let mut part = self.parts.borrow().get(pid).cloned().unwrap_or_default();

                        // update the partition id
                        new_part_keys.insert(partition_key, new_pid.clone());

                        // update the partition
                        part.insert(*k, v.to_string());
                        new_parts.insert(new_pid, part);
                    }
                }
                None => {
                    // we generate a new partition id
                    let new_pid = self.next_part_id(&partition_key);
                    // and update the partition id
                    let old_pid = new_part_keys.insert(partition_key, new_pid.clone());
                    let mut new_values = old_pid
                        .map(|pid| self.parts.borrow().get(&pid).cloned().unwrap_or_default())
                        .unwrap_or_default();
                    new_values.insert(*k, v.to_string());
                    new_parts.insert(new_pid, new_values);
                }
            }
        });

        new_parts.iter().for_each(|(pid, values)| {
            self.save_partition(pid, values);
        });

        let from = self.part_keys.clone();

        let res = self.node.borrow_mut().sync_rpc(
            SVC,
            MessageExtra::KvCas(KvCasData {
                key: DB_PARTITION_KEY.into(),
                from: serde_json::to_value(from).unwrap(),
                to: serde_json::to_value(new_part_keys).unwrap(),
                create_if_not_exists: true,
            }),
        );
        match res {
            MessageExtra::KvCasOk => true,
            // if cas fails, tell the client
            MessageExtra::Error(_) => false,
            _ => panic!("wrong response for lin-kv cas"),
        }
    }

    fn get(&self, key: &usize) -> Option<String> {
        let idx = self.part_key(key);
        self.part_keys.get(&idx).and_then(|id| {
            let mut parts_ref = self.parts.borrow_mut();
            match parts_ref.get(id) {
                Some(part) => part.get(&key).cloned(),
                None => {
                    let partition = self.load_partition(&id);
                    eprintln!("partition: {partition:#?}");
                    let res = partition.get(key).cloned();
                    parts_ref.insert(id.to_string(), partition);
                    res
                }
            }
        })
    }

    fn contains_key(&self, key: &usize) -> bool {
        self.get(key).is_some()
    }
}

pub struct Transactor {
    node: Rc<RefCell<Node>>,
}

impl Transactor {
    pub fn new(node: &Rc<RefCell<Node>>) -> Self {
        Self { node: node.clone() }
    }

    pub fn load_chunk(&self, chunk_id: &str) -> Vec<usize> {
        let res = self.node.borrow_mut().sync_rpc(
            SVC,
            MessageExtra::KvRead(KvReadExtra {
                key: chunk_id.into(),
            }),
        );

        match res {
            MessageExtra::KvReadOk(v) => serde_json::from_value(v.value).unwrap(),
            MessageExtra::Error(err) => {
                if err.code == 20 {
                    Vec::new()
                } else {
                    panic!("wrong response for lin-kv read")
                }
            }
            _ => panic!("wrong response for lin-kv read"),
        }
    }

    pub fn save_chunk(&self, chunk_id: &str, values: &[usize]) {
        let res = self.node.borrow_mut().sync_rpc(
            SVC,
            MessageExtra::KvWrite(KvWriteExtra {
                key: chunk_id.into(),
                value: serde_json::to_value(values).unwrap(),
            }),
        );
        if !matches!(res, MessageExtra::KvWriteOk) {
            unreachable!("the write error is not expected");
        }
    }

    pub fn next_id(&self) -> usize {
        static mut NEXT_ID: usize = 0;
        unsafe {
            NEXT_ID += 1;
        }
        unsafe { NEXT_ID }
    }

    pub fn new_thunk_id(&self) -> String {
        format!("{}-{}", self.node.borrow().id, self.next_id())
    }

    pub fn transact(&mut self, txns: &[Query]) -> Result<Vec<Query>, ErrorExtra> {
        let keys = { txns.iter().map(|txn| txn.1).collect::<HashSet<_>>() };
        eprintln!("keys: {keys:?}");

        let root = Root::load(&self.node);

        // loadPartialState
        let state: HashMap<usize, Vec<usize>> = keys
            .iter()
            .filter(|x| root.contains_key(&x))
            .map(|x| {
                let id = root.get(x).unwrap();
                (*x, self.load_chunk(&id))
            })
            .collect();
        eprintln!("state: {state:#?}");

        let (state2, txn2) = {
            let mut state2 = state.clone();
            let txn2 = txns
                .iter()
                .map(|x| {
                    let Query(op, k, qv) = x;
                    match op.as_str() {
                        "r" => {
                            let v = state2.get(k).cloned();
                            Query(op.clone(), *k, QueryValue::Read(v))
                        }
                        "append" => {
                            let QueryValue::Append(value) = qv else {
                                panic!("wrong value for 'append' operation");
                            };
                            state2
                                .entry(*k)
                                .and_modify(|v| v.push(*value))
                                .or_insert(vec![*value]);
                            x.clone()
                        }
                        _ => panic!("wrong operation"),
                    }
                })
                .collect();
            (state2, txn2)
        };
        eprintln!("state2: {state2:#?}");

        let write_keys = {
            txns.iter()
                .filter(|x| x.0 == "append")
                .map(|x| x.1)
                .collect::<HashSet<_>>()
        };

        // savePartialState
        let saved: HashMap<usize, String> = write_keys
            .iter()
            .map(|k| {
                let thunk_id = self.new_thunk_id();
                let thunk_values = state2.get(k).cloned().unwrap_or_default();
                self.save_chunk(&thunk_id, &thunk_values);
                (*k, thunk_id)
            })
            .collect();

        let ok = root.save(&saved);

        if ok {
            Ok(txn2)
        } else {
            Err(ErrorExtra {
                code: 30,
                text: "root altered".to_string(),
            })
        }
    }
}
