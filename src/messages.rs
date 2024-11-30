use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::HashSet;

/// protocol specification from https://github.com/jepsen-io/maelstrom/blob/main/doc/protocol.md
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    pub src: String,
    pub dest: String,
    pub body: MessageBody,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MessageBody {
    // reference: https://serde.rs/field-attrs.html
    #[serde(skip_serializing_if = "Option::is_none")]
    pub msg_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_reply_to: Option<u64>,

    // https://serde.rs/attr-flatten.html
    // The flatten attribute inlines keys from a field into the parent struct.
    #[serde(flatten)]
    pub extra: MessageExtra,
}

/// protocol: https://github.com/jepsen-io/maelstrom/blob/main/doc/protocol.md
#[derive(Serialize, Deserialize, Debug, Clone)]
// https://serde.rs/container-attrs.html
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum MessageExtra {
    Error(ErrorExtra),
    Init(InitRequestExtra),
    InitOk,
    Echo(EchoRequestExtra),
    EchoOk(EchoResponseExtra),
    Generate,
    GenerateOk(GenerateResponseExtra),
    Topology(TopologyRequestExtra),
    TopologyOk,
    Broadcast(BroadcastRequestExtra),
    BroadcastOk,
    #[cfg(not(feature = "lin_kv"))]
    Read,
    #[cfg(not(feature = "lin_kv"))]
    ReadOk(ReadResponseExtra),
    Txn(TxnRequestExtra),
    TxnOk(TxnResponseExtra),
    #[cfg(feature = "lin_kv")]
    #[serde(rename = "read")]
    KvRead(KvReadExtra),
    #[cfg(feature = "lin_kv")]
    #[serde(rename = "read_ok")]
    KvReadOk(KvReadOkExtra),
    #[serde(rename = "write")]
    KvWrite(KvWriteExtra),
    #[serde(rename = "write_ok")]
    KvWriteOk,
    #[serde(rename = "cas")]
    KvCas(KvCasData),
    #[serde(rename = "cas_ok")]
    KvCasOk,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ErrorExtra {
    pub code: u64,
    pub text: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KvReadExtra {
    pub key: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KvReadOkExtra {
    pub value: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KvWriteExtra {
    pub key: usize,
    pub value: usize,
}

/// Atomically compare-and-sets a single key:
/// if the value of key is currently `from`, sets it to `to`.
/// Returns error 20 if the key doesn't exist, and 22 if the
/// from value doesn't match.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KvCasData {
    pub key: serde_json::Value,
    pub from: serde_json::Value,
    pub to: serde_json::Value,
    pub create_if_not_exists: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EchoRequestExtra {
    pub echo: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EchoResponseExtra {
    pub echo: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InitRequestExtra {
    pub node_id: String,
    pub node_ids: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GenerateResponseExtra {
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TopologyRequestExtra {
    pub topology: HashMap<String, Vec<String>>,
}

pub type BroadcastValue = u64;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BroadcastRequestExtra {
    pub message: BroadcastValue,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReadResponseExtra {
    pub messages: HashSet<BroadcastValue>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TxnRequestExtra {
    pub txn: Vec<Query>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TxnResponseExtra {
    pub txn: Vec<Query>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Query(pub String, pub usize, pub QueryValue);

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum QueryValue {
    Read(Option<Vec<usize>>),
    Append(usize),
}
