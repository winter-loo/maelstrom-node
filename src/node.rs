use crate::message_handlers::*;
use crate::messages::*;
use std::collections::HashMap;
use std::collections::HashSet;

pub struct Node {
    pub id: String,
    pub node_ids: Vec<String>,
    pub topology: HashMap<String, Vec<String>>,
    pub messages_seen: HashSet<BroadcastValue>,
}

impl Node {
    pub fn new() -> Self {
        Node {
            id: String::from(""),
            node_ids: Vec::new(),
            topology: HashMap::new(),
            messages_seen: HashSet::new(),
        }
    }

    pub fn handle_message(
        &mut self,
        req: &Message,
        msg_handlers: &[Box<dyn MessageHandler>],
    ) -> Message {
        let mut res = Message {
            src: req.dest.clone(),
            dest: req.src.clone(),
            body: MessageBody {
                msg_id: None,
                in_reply_to: None,
                extra: MessageExtra::Empty,
            },
        };
        res.body.in_reply_to = req.body.msg_id;
        if let Some(msg_id) = req.body.msg_id {
            res.body.msg_id = Some(msg_id + 1);
        }

        let extra = &req.body.extra;
        let handler = msg_handlers.iter().find(|h| h.can_handle(extra));

        match handler {
            Some(handler) => {
                res.body.extra = handler.handle(self, extra).unwrap();
            }
            None => {
                eprintln!("unknow message: {:#?}", req);
            }
        }

        res
    }

    pub fn send(&self, res: Message) {
        println!("{}", serde_json::to_string(&res).unwrap());
    }
}
