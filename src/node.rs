use std::collections::HashMap;
use crate::messages::*;
use uuid::Uuid;

#[derive(Debug)]
pub struct Node {
    pub id: String,
    pub node_ids: Vec<String>,
    pub topology: HashMap<String, Vec<String>>,
    pub messages_seen: Vec<BroadcastValue>,
}

impl Node {
    pub fn new() -> Self {
        Node {
            id: String::from(""),
            node_ids: Vec::new(),
            topology: HashMap::new(),
            messages_seen: Vec::new(),
        }
    }

    pub fn handle_message(&mut self, req: Message) -> Message {
        let mut res = Message {
            src: req.dest,
            dest: req.src,
            body: MessageBody {
                msg_id: None,
                in_reply_to: None,
                payload: Payload::Empty,
            },
        };
        res.body.in_reply_to = req.body.msg_id;
        if let Some(msg_id) = req.body.msg_id {
            res.body.msg_id = Some(msg_id + 1);
        }

        match req.body.payload {
            Payload::Empty => {}

            Payload::Init(init) => {
                self.id = init.node_id;
                self.node_ids = init.node_ids;

                res.body.payload = Payload::InitOk;
            }
            Payload::InitOk => {}

            Payload::Echo(payload) => {
                res.body.payload = Payload::EchoOk(EchoResponsePayload { echo: payload.echo });
            }
            Payload::EchoOk(_) => {}

            Payload::Generate => {
                res.body.payload = Payload::GenerateOk(GenerateResponsePayload {
                    id: Uuid::new_v4().to_string(),
                });
            }
            Payload::GenerateOk(_) => {}

            Payload::Topology(payload) => {
                res.body.payload = Payload::TopologyOk;
                self.topology = payload.topology;
            }
            Payload::TopologyOk => {},

            Payload::Broadcast(payload) => {
                res.body.payload = Payload::BroadcastOk;
                self.messages_seen.push(payload.message);

                for neibor in self.topology.get(&self.id).unwrap() {
                    let my_req = Message {
                        src: self.id.clone(),
                        dest: neibor.clone(),
                        body: MessageBody {
                            msg_id: Some(1),
                            in_reply_to: None,
                            payload: Payload::Broadcast(payload.clone()),
                        }
                    };
                    self.send(my_req);
                }
            }
            Payload::BroadcastOk => {},

            Payload::Read => {
                res.body.payload = Payload::ReadOk(ReadResponsePayload{
                    messages: self.messages_seen.clone(),
                });
            }
            Payload::ReadOk(_) => {},
        }

        res
    }

    pub fn send(&self, req: Message) {
        println!("{}", serde_json::to_string(&req).unwrap());
    }
}
