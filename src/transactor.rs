//! read and write to lin-kv
use crate::messages::*;
use crate::node::Node;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const DB_KEY: &str = "ROOT";

pub struct Transactor<'a> {
    node: &'a mut Node,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Database {
    /// Why not use HashMap<usize, Vec<usize>>?
    /// The problem is that String cannot be implicitly converted to usize,
    /// otherwise we need to implement Custom Serialize and Deserialize.
    #[serde(flatten)]
    inner: HashMap<String, Vec<usize>>,
}

impl Database {
    pub fn transact(&self, txns: &[Query]) -> (Database, Vec<Query>) {
        let mut results = Vec::new();
        let mut new_db = self.inner.clone();

        for txn in txns {
            let Query(op, key, value) = txn;
            eprintln!("op: {op}, key: {key}, value: {value:?}");
            match op.as_str() {
                "r" => {
                    let old_values = new_db.get(&key.to_string()).cloned().unwrap_or(vec![]);
                    results.push(Query(
                        op.clone(),
                        key.clone(),
                        QueryValue::Read(Some(old_values)),
                    ));
                }
                "append" => {
                    results.push(txn.clone());

                    let value = match value {
                        QueryValue::Append(v) => v,
                        QueryValue::Read(_) => panic!("wrong value for 'append' operation"),
                    };

                    new_db.entry(key.to_string()).or_insert_with(|| vec![]).push(*value);
                }
                _ => {}
            }
        }
        (Database { inner: new_db }, results)
    }

    pub fn from_json_value(json: serde_json::Value) -> Self {
        serde_json::from_value(json).unwrap()
    }

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap()
    }
}

impl<'a> Transactor<'a> {
    pub fn new(node: &'a mut Node) -> Self {
        Self { node }
    }

    /// send a read request to lin-kv
    pub fn kv_read(&mut self, key: &serde_json::Value) -> serde_json::Value {
        let res =self.node.sync_rpc(Message {
            src: self.node.id.clone(),
            dest: "lin-kv".to_string(),
            body: MessageBody {
                msg_id: Some(self.node.next_msg_id()),
                in_reply_to: None,
                extra: MessageExtra::KvRead(KvReadExtra { key: key.clone() }),
            },
        });
        let old_values = match res.body.extra {
            MessageExtra::KvReadOk(v) => v.value,
            MessageExtra::Error(err) => {
                if err.code == 20 {
                    Database { inner: HashMap::new() }.to_json_value()
                } else {
                    panic!("wrong response for lin-kv read")
                }
            }
            _ => panic!("wrong response for lin-kv read"),
        };
        old_values
    }

    /// send a CAS request to lin-kv
    pub fn kv_cas(&mut self, key: &serde_json::Value, from: &serde_json::Value, to: &serde_json::Value) -> MessageExtra {
        let res = self.node.sync_rpc(Message {
            src: self.node.id.clone(),
            dest: "lin-kv".to_string(),
            body: MessageBody {
                msg_id: Some(self.node.next_msg_id()),
                in_reply_to: None,
                extra: MessageExtra::KvCas(KvCasData {
                    key: key.clone(),
                    from: from.clone(),
                    to: to.clone(),
                    create_if_not_exists: true,
                }),
            },
        });

        res.body.extra
    }

    /// perform a list of read and write operations
    pub fn transact(&mut self, txns: &[Query]) -> Result<Vec<Query>, ErrorExtra> {
        let db_key = serde_json::Value::String(DB_KEY.to_string());
        // Load the current value from lin-kv
        let current_db = Database::from_json_value(self.kv_read(&db_key));
        eprintln!("current db: {:#?}", current_db.inner);
        // Apply txn
        let (next_db, txns) = current_db.transact(txns);
        eprintln!("next db: {:#?}", next_db.inner);
        // Save resulting state iff it hasn't changed
        let res =self.kv_cas(&db_key, &current_db.to_json_value(), &next_db.to_json_value());
        match res {
            MessageExtra::KvCasOk => Ok(txns),
            // if cas fails, tell the client
            MessageExtra::Error(err) => Err(err),
            _ => panic!("wrong response for lin-kv cas"),
        }
    }
}
