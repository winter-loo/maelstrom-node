use crate::messages::*;
use crate::node::*;
use uuid::Uuid;

pub trait MessageHandler {
    fn can_handle(&self, req: &Message) -> bool;

    /// Each Message may or may not mutate the state of a Node.
    /// When None is returned, the message is not responded to its peer.
    fn handle(&self, node: &mut Node, req: &Message) -> Option<MessageExtra>;
}

pub struct InitHandler;

impl MessageHandler for InitHandler {
    fn can_handle(&self, req: &Message) -> bool {
        matches!(req.body.extra, MessageExtra::Init(_))
    }

    fn handle(&self, node: &mut Node, req: &Message) -> Option<MessageExtra> {
        if let MessageExtra::Init(init) = &req.body.extra {
            node.id = init.node_id.clone();
            node.node_ids = init.node_ids.clone();

            Some(MessageExtra::InitOk)
        } else {
            None
        }
    }
}

pub struct InitOkHandler;

impl MessageHandler for InitOkHandler {
    fn can_handle(&self, req: &Message) -> bool {
        matches!(req.body.extra, MessageExtra::InitOk)
    }

    fn handle(&self, _node: &mut Node, _req: &Message) -> Option<MessageExtra> {
        None
    }
}

pub struct EchoHandler;

impl MessageHandler for EchoHandler {
    fn can_handle(&self, req: &Message) -> bool {
        matches!(req.body.extra, MessageExtra::Echo(_))
    }

    fn handle(&self, _node: &mut Node, req: &Message) -> Option<MessageExtra> {
        if let MessageExtra::Echo(echo) = &req.body.extra {
            Some(MessageExtra::EchoOk(EchoResponseExtra {
                echo: echo.echo.clone(),
            }))
        } else {
            None
        }
    }
}

pub struct EchoOkHandler;

impl MessageHandler for EchoOkHandler {
    fn can_handle(&self, req: &Message) -> bool {
        matches!(req.body.extra, MessageExtra::EchoOk(_))
    }

    fn handle(&self, _node: &mut Node, _req: &Message) -> Option<MessageExtra> {
        None
    }
}

pub struct GenerateHandler;

impl MessageHandler for GenerateHandler {
    fn can_handle(&self, req: &Message) -> bool {
        matches!(req.body.extra, MessageExtra::Generate)
    }

    fn handle(&self, _node: &mut Node, req: &Message) -> Option<MessageExtra> {
        if let MessageExtra::Generate = &req.body.extra {
            Some(MessageExtra::GenerateOk(GenerateResponseExtra {
                id: Uuid::new_v4().to_string(),
            }))
        } else {
            None
        }
    }
}

pub struct GenerateOkHandler;

impl MessageHandler for GenerateOkHandler {
    fn can_handle(&self, req: &Message) -> bool {
        matches!(req.body.extra, MessageExtra::GenerateOk(_))
    }

    fn handle(&self, _node: &mut Node, _req: &Message) -> Option<MessageExtra> {
        None
    }
}

pub struct TopologyHandler;

impl MessageHandler for TopologyHandler {
    fn can_handle(&self, req: &Message) -> bool {
        matches!(req.body.extra, MessageExtra::Topology(_))
    }

    fn handle(&self, node: &mut Node, req: &Message) -> Option<MessageExtra> {
        if let MessageExtra::Topology(payload) = &req.body.extra {
            node.topology = payload.topology.clone();
            Some(MessageExtra::TopologyOk)
        } else {
            None
        }
    }
}

pub struct TopologyOkHandler;

impl MessageHandler for TopologyOkHandler {
    fn can_handle(&self, req: &Message) -> bool {
        matches!(req.body.extra, MessageExtra::TopologyOk)
    }

    fn handle(&self, _node: &mut Node, _req: &Message) -> Option<MessageExtra> {
        None
    }
}

pub struct BroadcastHandler;

impl MessageHandler for BroadcastHandler {
    fn can_handle(&self, req: &Message) -> bool {
        matches!(req.body.extra, MessageExtra::Broadcast(_))
    }

    fn handle(&self, node: &mut Node, req: &Message) -> Option<MessageExtra> {
        if let MessageExtra::Broadcast(payload) = &req.body.extra {
            if node.messages_seen.insert(payload.message) {
                for neibor in node.topology.get(&node.id).unwrap() {
                    if neibor == &req.src {
                        continue;
                    }
                    let my_req = Message {
                        src: node.id.clone(),
                        dest: neibor.clone(),
                        body: MessageBody {
                            // Per the doc https://github.com/jepsen-io/maelstrom/blob/main/doc/03-broadcast/01-broadcast.md,
                            // Inter-server messages don't have a msg_id, and don't need a response.
                            msg_id: None,
                            in_reply_to: None,
                            // broadcast message
                            extra: req.body.extra.clone(),
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

pub struct BroadcastOkHandler;

impl MessageHandler for BroadcastOkHandler {
    fn can_handle(&self, req: &Message) -> bool {
        matches!(req.body.extra, MessageExtra::BroadcastOk)
    }

    fn handle(&self, _node: &mut Node, _req: &Message) -> Option<MessageExtra> {
        None
    }
}

pub struct ReadHandler;

impl MessageHandler for ReadHandler {
    fn can_handle(&self, req: &Message) -> bool {
        matches!(req.body.extra, MessageExtra::Read)
    }

    fn handle(&self, node: &mut Node, req: &Message) -> Option<MessageExtra> {
        if let MessageExtra::Read = &req.body.extra {
            Some(MessageExtra::ReadOk(ReadResponseExtra {
                messages: node.messages_seen.clone(),
            }))
        } else {
            None
        }
    }
}

pub struct ReadOkHandler;

impl MessageHandler for ReadOkHandler {
    fn can_handle(&self, req: &Message) -> bool {
        matches!(req.body.extra, MessageExtra::ReadOk(_))
    }

    fn handle(&self, _node: &mut Node, _req: &Message) -> Option<MessageExtra> {
        None
    }
}