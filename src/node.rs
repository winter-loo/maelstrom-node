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
    ) -> Option<Message> {
        let extra = &req.body.extra;
        if let Some(extra) = extra {
            let handler = msg_handlers.iter().find(|h| h.can_handle(extra));

            match handler {
                Some(handler) => {
                    let res_extra = handler.handle(self, extra);
                    
                    // Per the doc https://github.com/jepsen-io/maelstrom/blob/main/doc/03-broadcast/01-broadcast.md,
                    // Inter-server messages don't have a msg_id, and don't need a response.
                    if req.body.msg_id.is_none() || res_extra.is_none() {
                        None
                    } else {
                        let mut res = Message {
                            src: req.dest.clone(),
                            dest: req.src.clone(),
                            body: MessageBody {
                                msg_id: None,
                                in_reply_to: None,
                                extra: None,
                            },
                        };
                        res.body.in_reply_to = req.body.msg_id;
                        if let Some(msg_id) = req.body.msg_id {
                            res.body.msg_id = Some(msg_id + 1);
                        }
                        res.body.extra = res_extra;
                        Some(res)
                    }
                }
                None => {
                    eprintln!("unknow message: {:#?}", req);
                    None
                }
            }
        } else {
            None
        }
    }

    pub fn send(&self, res: Message) {
        println!("{}", serde_json::to_string(&res).unwrap());
    }
}
