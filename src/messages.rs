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
    pub payload: Payload,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
// https://serde.rs/container-attrs.html
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum Payload {
    Empty,
    Init(InitRequestPayload),
    InitOk,
    Echo(EchoRequestPayload),
    EchoOk(EchoResponsePayload),
    Generate,
    GenerateOk(GenerateResponsePayload),
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct EchoRequestPayload {
    pub echo: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct EchoResponsePayload {
    pub echo: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct InitRequestPayload {
    pub node_id: String,
    pub node_ids: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GenerateResponsePayload {
    pub id: String,
}
