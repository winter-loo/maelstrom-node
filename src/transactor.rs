//! read and write to lin-kv
use crate::messages::*;
use crate::node::Node;

pub struct Transactor<'a> {
    node: &'a mut Node,
}

impl<'a> Transactor<'a> {
    pub fn new(node: &'a mut Node) -> Self {
        Self { node }
    }

    /// send a read request to lin-kv
    pub fn kv_read(&mut self, key: &str) -> Vec<u64> {
        let res =self.node.sync_rpc(Message {
            src: self.node.id.clone(),
            dest: "lin-kv".to_string(),
            body: MessageBody {
                msg_id: Some(self.node.next_msg_id()),
                in_reply_to: None,
                extra: MessageExtra::KvRead(KvReadExtra { key: key.to_string() }),
            },
        });
        let old_values = match res.body.extra {
            MessageExtra::KvReadOk(values) => values.value,
            MessageExtra::Error(err) => {
                if err.code == 20 {
                    vec![]
                } else {
                    panic!("wrong response for lin-kv read")
                }
            }
            _ => panic!("wrong response for lin-kv read"),
        };
        old_values
    }

    /// send a CAS request to lin-kv
    pub fn kv_cas(&mut self, key: &str, from: &Vec<u64>, to: &Vec<u64>) {
        let res = self.node.sync_rpc(Message {
            src: self.node.id.clone(),
            dest: "lin-kv".to_string(),
            body: MessageBody {
                msg_id: Some(self.node.next_msg_id()),
                in_reply_to: None,
                extra: MessageExtra::KvCas(KvCasData {
                    key: key.to_string(),
                    from: from.clone(),
                    to: to.clone(),
                    create_if_not_exists: true,
                }),
            },
        });

        if !matches!(res.body.extra, MessageExtra::KvCasOk) {
            eprintln!("CAS of {key} failed");
        }
    }

    /// perform a list of read and write operations
    pub fn transact(&mut self, txns: &[Query]) -> Vec<Query> {
        let mut results = Vec::new();
        for txn in txns {
            let Query(op, key, value) = txn;
            eprintln!("op: {op}, key: {key}, value: {value:?}");
            match op.as_str() {
                "r" => {
                    let old_values = self.kv_read(key);
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

                    let old_values = self.kv_read(key);
                    let mut new_values = old_values.clone();
                    new_values.push(value.clone());

                    self.kv_cas(key, &old_values, &new_values);
                }
                _ => {}
            }
        }
        results
    }
}
