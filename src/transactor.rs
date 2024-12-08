//! read and write to lin-kv
use crate::messages::*;
use crate::node::Node;
use crate::thunk::{LazyValue, Thunk};
use serde_json::json;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

const DB_KEY: &str = "ROOT";

pub struct Transactor {
    node: Rc<RefCell<Node>>,
}

#[derive(Debug, Clone)]
pub struct Database {
    // Thunk will use its own id to get the value, i.e. HaspMap<String, Thunk<Vec<usize>>>,
    // from lin-kv store which should response with {"k1": "id1", "k2": "id2"}.
    // data flow:
    // root -> id(database pointer)
    // id -> value({"k1": "v1", "k2": "v2"}, should be deserialized to HashMap<String, Thunk<Vec<usize>>>)
    // v1 is an ID of inner thunk
    // v1 -> value([1,2,3])
    inner: Thunk<HashMap<usize, Thunk<Vec<usize>>>>,
    node: Rc<RefCell<Node>>,
}

impl Database {
    pub fn transact(&mut self, txns: &[Query]) -> Vec<Query> {
        let mut results = Vec::new();
        let mut new_map: HashMap<usize, Thunk<Vec<usize>>> = HashMap::new();

        // new db contains only the keys we need for the txns
        // instead of the whole database keys
        let om = self.inner.value();
        eprintln!("inner value: {:#?}", om);
        if let Some(om) = om {
            for txn in txns {
                let v = om.get(&txn.1);
                if let Some(v) = v {
                    let mut v = v.clone();
                    v.node = self.node.clone();
                    new_map.insert(txn.1, v);
                }
            }
            eprintln!("new map initialized: {:#?}", new_map);
        }

        for txn in txns {
            let Query(op, key, value) = txn;
            eprintln!("op: {op}, key: {key}, value: {value:?}");
            match op.as_str() {
                "r" => {
                    let old_values = {
                        let old_thunk = new_map.get(key);
                        match old_thunk {
                            Some(t) => t.value(),
                            None => None,
                        }
                    };
                    results.push(Query(op.clone(), *key, QueryValue::Read(old_values)));
                }
                "append" => {
                    results.push(txn.clone());

                    let value = match value {
                        QueryValue::Append(v) => *v,
                        QueryValue::Read(_) => panic!("wrong value for 'append' operation"),
                    };
                    let thunk = new_map.entry(*key).or_insert_with(|| Thunk {
                        node: self.node.clone(),
                        id: RefCell::new(LazyValue::UnLoaded),
                        value: RefCell::new(LazyValue::Loaded(vec![])),
                        dirty: RefCell::new(false),
                    });
                    // update thunk
                    let mut old_values = thunk.value().unwrap_or(vec![]);
                    old_values.push(value);
                    thunk.set_value(old_values);
                    eprintln!("thunk2: {:#?}", thunk);
                }
                _ => unimplemented!(),
            }
        }

        self.merge(new_map);

        results
    }

    pub fn merge(&mut self, new_map: HashMap<usize, Thunk<Vec<usize>>>) {
        eprintln!("merging new map: {:#?}", new_map);
        eprintln!("self inner: {:#?}", self.inner);
        let new_map2 = new_map.clone();
        let merged = self.inner.merge(|old| {
            let mut changed = false;
            for (k, v) in new_map2 {
                if v.dirty() {
                    old.insert(k, v);
                    changed = true;
                }
            }
            eprintln!("merged map: {:#?}", old);
            changed
        });
        if !merged {
            self.inner.set_value(new_map);
        }
        eprintln!("updated db: {:#?}", self.inner);
    }

    pub fn from_json_value(node: Rc<RefCell<Node>>, json: serde_json::Value) -> Self {
        let root_id = serde_json::from_value(json);
        eprintln!("root id: {root_id:?}");
        let inner = match root_id {
            Ok(id) => Thunk {
                node: node.clone(),
                id: RefCell::new(LazyValue::Loaded(id)),
                // this will be from json { "k1": "id1", "k2": "id2" }
                value: RefCell::new(LazyValue::UnLoaded),
                dirty: RefCell::new(false),
            },
            Err(_) => Thunk {
                node: node.clone(),
                id: RefCell::new(LazyValue::LoadFailed),
                value: RefCell::new(LazyValue::LoadFailed),
                dirty: RefCell::new(false),
            },
        };

        Database { inner, node }
    }

    pub fn to_json_value(&self) -> serde_json::Value {
        let inner = self.inner.id.clone();
        serde_json::to_value(&inner).unwrap()
    }

    pub fn save(&mut self) {
        let m = self.inner.value();
        if let Some(m) = m {
            m.iter().for_each(|(_, v)| {
                eprintln!("saving inner thunk: {:?}", v);
                let _ = v.save();
            });
            eprintln!("saving outer thunk: {:?}", self.inner);
            let _ = self.inner.save();
        }
    }
}

impl Transactor {
    const SVC: &'static str = "lin-kv";

    pub fn new(node: &Rc<RefCell<Node>>) -> Self {
        Self { node: node.clone() }
    }

    /// send a read request to lin-kv
    pub fn kv_read(&mut self, key: &serde_json::Value) -> serde_json::Value {
        let res = self.node.borrow_mut().sync_rpc(
            Self::SVC,
            MessageExtra::KvRead(KvReadExtra { key: key.clone() }),
        );

        match res {
            MessageExtra::KvReadOk(v) => v.value,
            MessageExtra::Error(err) => {
                if err.code == 20 {
                    serde_json::Value::Null
                } else {
                    panic!("wrong response for lin-kv read")
                }
            }
            _ => panic!("wrong response for lin-kv read"),
        }
    }

    /// send a CAS request to lin-kv
    pub fn kv_cas(
        &mut self,
        key: &serde_json::Value,
        from: &serde_json::Value,
        to: &serde_json::Value,
    ) -> MessageExtra {
        eprintln!("kv_cas key: {key}, from: {from}, to: {to}");
        let res = self.node.borrow_mut().sync_rpc(
            Self::SVC,
            MessageExtra::KvCas(KvCasData {
                key: key.clone(),
                from: from.clone(),
                to: to.clone(),
                create_if_not_exists: true,
            }),
        );

        res
    }

    /// perform a list of read and write operations
    pub fn transact(&mut self, txns: &[Query]) -> Result<Vec<Query>, ErrorExtra> {
        let db_key = json!(DB_KEY);
        // Load the current value from lin-kv
        let id1 = self.kv_read(&db_key);
        eprintln!("root id: {id1:?}");
        let mut current_db = Database::from_json_value(self.node.clone(), id1);
        eprintln!("current db status: {:#?}", current_db.inner);
        let old_id = current_db.to_json_value();
        // Apply txn
        let txns = current_db.transact(txns);
        eprintln!("next db status: {:#?}", current_db.inner);
        let new_id = current_db.to_json_value();

        // Save resulting state iff it hasn't changed
        if old_id == new_id {
            return Ok(txns);
        }

        current_db.save();
        match self.kv_cas(&db_key, &old_id, &new_id) {
            MessageExtra::KvCasOk => Ok(txns),
            // if cas fails, tell the client
            MessageExtra::Error(err) => Err(err),
            _ => panic!("wrong response for lin-kv cas"),
        }
    }
}
