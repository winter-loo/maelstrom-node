use crate::messages::{KvCasData, KvWriteExtra, QueryValue};
use crate::{
    messages::{ErrorExtra, KvReadExtra, MessageExtra, Query},
    node::Node,
};
use std::collections::{HashMap, HashSet};
use std::{cell::RefCell, rc::Rc};

const DB_KEY: &str = "ROOT";

pub struct Transactor {
    node: Rc<RefCell<Node>>,
}

impl Transactor {
    const SVC: &str = "lin-kv";

    pub fn new(node: &Rc<RefCell<Node>>) -> Self {
        Self { node: node.clone() }
    }

    /// send a read request to lin-kv
    pub fn get_root(&mut self) -> HashMap<usize, String> {
        let res = self.node.borrow_mut().sync_rpc(
            Self::SVC,
            MessageExtra::KvRead(KvReadExtra { key: DB_KEY.into() }),
        );

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

    /// send a CAS request to lin-kv
    pub fn cas_root(
        &mut self,
        from: &HashMap<usize, String>,
        to: &HashMap<usize, String>,
    ) -> MessageExtra {
        eprintln!("kv_cas root, from: {from:#?}, to: {to:#?}");
        let res = self.node.borrow_mut().sync_rpc(
            Self::SVC,
            MessageExtra::KvCas(KvCasData {
                key: DB_KEY.into(),
                from: serde_json::to_value(from).unwrap(),
                to: serde_json::to_value(to).unwrap(),
                create_if_not_exists: true,
            }),
        );

        res
    }

    pub fn load_chunk(&self, chunk_id: &str) -> Vec<usize> {
        let res = self.node.borrow_mut().sync_rpc(
            Self::SVC,
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
            Self::SVC,
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

    pub fn merge(
        &self,
        old: &HashMap<usize, String>,
        new: &HashMap<usize, String>,
    ) -> HashMap<usize, String> {
        let mut res = old.clone();
        for (k, v) in new {
            res.insert(*k, v.clone());
        }
        res
    }

    pub fn transact(&mut self, txns: &[Query]) -> Result<Vec<Query>, ErrorExtra> {
        let keys = { txns.iter().map(|txn| txn.1).collect::<HashSet<_>>() };
        eprintln!("keys: {keys:?}");

        let root = self.get_root();
        eprintln!("root: {root:#?}");

        // loadPartialState
        let state: HashMap<usize, Vec<usize>> = keys
            .iter()
            .filter(|x| root.contains_key(&x))
            .map(|x| {
                let id = root.get(x).unwrap();
                (*x, self.load_chunk(id))
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

        let write_keys = {
            txns.iter()
                .filter(|x| x.0 == "append")
                .map(|x| x.1)
                .collect::<Vec<_>>()
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

        let root2 = self.merge(&root, &saved);

        let res = self.cas_root(&root, &root2);

        match res {
            MessageExtra::KvCasOk => Ok(txn2),
            // if cas fails, tell the client
            MessageExtra::Error(err) => Err(err),
            _ => panic!("wrong response for lin-kv cas"),
        }
    }
}

