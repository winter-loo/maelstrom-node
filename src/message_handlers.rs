use crate::messages::*;
use crate::node::*;
use uuid::Uuid;

pub trait MessageHandler {
    fn can_handle(&self, req: &MessageExtra) -> bool;

    /// each Message may or may not mutate the state of a Node.
    fn handle(&self, node: &mut Node, req: &MessageExtra) -> Option<MessageExtra>;
}

pub struct InitHandler;

impl MessageHandler for InitHandler {
    fn can_handle(&self, req: &MessageExtra) -> bool {
        matches!(req, MessageExtra::Init(_))
    }

    fn handle(&self, node: &mut Node, req: &MessageExtra) -> Option<MessageExtra> {
        if let MessageExtra::Init(init) = req {
            node.id = init.node_id.clone();
            node.node_ids = init.node_ids.clone();

            Some(MessageExtra::InitOk)
        } else {
            None
        }
    }
}

pub struct EchoHandler;

impl MessageHandler for EchoHandler {
    fn can_handle(&self, req: &MessageExtra) -> bool {
        matches!(req, MessageExtra::Echo(_))
    }

    fn handle(&self, _node: &mut Node, req: &MessageExtra) -> Option<MessageExtra> {
        if let MessageExtra::Echo(echo) = req {
            Some(MessageExtra::EchoOk(EchoResponseExtra {
                echo: echo.echo.clone(),
            }))
        } else {
            None
        }
    }
}

pub struct GenerateHandler;

impl MessageHandler for GenerateHandler {
    fn can_handle(&self, req: &MessageExtra) -> bool {
        matches!(req, MessageExtra::Generate)
    }

    fn handle(&self, _node: &mut Node, req: &MessageExtra) -> Option<MessageExtra> {
        if let MessageExtra::Generate = req {
            Some(MessageExtra::GenerateOk(GenerateResponseExtra {
                id: Uuid::new_v4().to_string(),
            }))
        } else {
            None
        }
    }
}

pub struct TopologyHandler;

impl MessageHandler for TopologyHandler {
    fn can_handle(&self, req: &MessageExtra) -> bool {
        matches!(req, MessageExtra::Topology(_))
    }

    fn handle(&self, node: &mut Node, req: &MessageExtra) -> Option<MessageExtra> {
        if let MessageExtra::Topology(payload) = req {
            node.topology = payload.topology.clone();
            Some(MessageExtra::TopologyOk)
        } else {
            None
        }
    }
}

pub struct BroadcastHandler;

impl MessageHandler for BroadcastHandler {
    fn can_handle(&self, req: &MessageExtra) -> bool {
        matches!(req, MessageExtra::Broadcast(_))
    }

    fn handle(&self, node: &mut Node, req: &MessageExtra) -> Option<MessageExtra> {
        if let MessageExtra::Broadcast(payload) = req {
            if node.messages_seen.insert(payload.message) {
                for neibor in node.topology.get(&node.id).unwrap() {
                    let my_req = Message {
                        src: node.id.clone(),
                        dest: neibor.clone(),
                        body: MessageBody {
                            msg_id: Some(1),
                            in_reply_to: None,
                            extra: req.clone(),
                        },
                    };
                    node.send(my_req);
                }
            }

            Some(MessageExtra::BroadcastOk)
        } else {
            None
        }
    }
}

pub struct ReadHandler;

impl MessageHandler for ReadHandler {
    fn can_handle(&self, req: &MessageExtra) -> bool {
        matches!(req, MessageExtra::Read)
    }

    fn handle(&self, node: &mut Node, req: &MessageExtra) -> Option<MessageExtra> {
        if let MessageExtra::Read = req {
            Some(MessageExtra::ReadOk(ReadResponseExtra {
                messages: node.messages_seen.clone(),
            }))
        } else {
            None
        }
    }
}
