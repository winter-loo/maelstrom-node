use std::collections::HashSet;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

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

#[derive(Serialize, Deserialize, Debug, Clone)]
// https://serde.rs/container-attrs.html
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum MessageExtra {
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
    Read,
    ReadOk(ReadResponseExtra),
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
