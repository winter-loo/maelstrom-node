use crate::messages::*;
use uuid::Uuid;

#[derive(Debug)]
pub struct Node {
    pub id: String,
    pub node_ids: Vec<String>,
}

impl Node {
    pub fn new() -> Self {
        Node {
            id: String::from(""),
            node_ids: Vec::new(),
        }
    }

    pub fn handle_message(&mut self, req: Message) -> String {
        let mut res = Message {
            src: req.dest,
            dest: req.src,
            body: MessageBody {
                msg_id: None,
                in_reply_to: None,
                payload: Payload::Empty,
            },
        };

        match req.body.payload {
            Payload::Empty => {}

            Payload::Init(init) => {
                self.id = init.node_id;
                self.node_ids = init.node_ids;

                res.body.in_reply_to = req.body.msg_id;
                res.body.payload = Payload::InitOk;
            }
            Payload::InitOk => {}

            Payload::Echo(payload) => {
                res.body.in_reply_to = req.body.msg_id;
                res.body.payload = Payload::EchoOk(EchoResponsePayload { echo: payload.echo });
            }
            Payload::EchoOk(_) => {}

            Payload::Generate => {
                res.body.in_reply_to = req.body.msg_id;
                res.body.payload = Payload::GenerateOk(GenerateResponsePayload {
                    id: Uuid::new_v4().to_string(),
                });
            }
            Payload::GenerateOk(_) => {}
        }

        serde_json::to_string(&res).unwrap()
    }
}
