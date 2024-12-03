use crate::messages::*;
use crate::node::*;
#[cfg(feature = "lin_kv")]
use crate::transactor::Transactor;
use std::cell::RefCell;
use std::rc::Rc;
use uuid::Uuid;

pub trait MessageHandler {
    fn can_handle(&self, req: &Message) -> bool;

    /// Each Message may or may not mutate the state of a Node.
    /// When None is returned, the message is not responded to its peer.
    fn handle(&self, node: &Rc<RefCell<Node>>, req: &Message) -> Option<MessageExtra>;
}

pub struct InitHandler;

impl MessageHandler for InitHandler {
    fn can_handle(&self, req: &Message) -> bool {
        matches!(req.body.extra, MessageExtra::Init(_))
    }

    fn handle(&self, node: &Rc<RefCell<Node>>, req: &Message) -> Option<MessageExtra> {
        if let MessageExtra::Init(init) = &req.body.extra {
            node.borrow_mut().id = init.node_id.clone();
            node.borrow_mut().node_ids = init.node_ids.clone();

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

    fn handle(&self, _node: &Rc<RefCell<Node>>, _req: &Message) -> Option<MessageExtra> {
        None
    }
}

pub struct EchoHandler;

impl MessageHandler for EchoHandler {
    fn can_handle(&self, req: &Message) -> bool {
        matches!(req.body.extra, MessageExtra::Echo(_))
    }

    fn handle(&self, _node: &Rc<RefCell<Node>>, req: &Message) -> Option<MessageExtra> {
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

    fn handle(&self, _node: &Rc<RefCell<Node>>, _req: &Message) -> Option<MessageExtra> {
        None
    }
}

pub struct GenerateHandler;

impl MessageHandler for GenerateHandler {
    fn can_handle(&self, req: &Message) -> bool {
        matches!(req.body.extra, MessageExtra::Generate)
    }

    fn handle(&self, _node: &Rc<RefCell<Node>>, req: &Message) -> Option<MessageExtra> {
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

    fn handle(&self, _node: &Rc<RefCell<Node>>, _req: &Message) -> Option<MessageExtra> {
        None
    }
}

pub struct TopologyHandler;

impl MessageHandler for TopologyHandler {
    fn can_handle(&self, req: &Message) -> bool {
        matches!(req.body.extra, MessageExtra::Topology(_))
    }

    fn handle(&self, node: &Rc<RefCell<Node>>, req: &Message) -> Option<MessageExtra> {
        if let MessageExtra::Topology(payload) = &req.body.extra {
            node.borrow_mut().topology = payload.topology.clone();
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

    fn handle(&self, _node: &Rc<RefCell<Node>>, _req: &Message) -> Option<MessageExtra> {
        None
    }
}

pub struct BroadcastHandler;

impl MessageHandler for BroadcastHandler {
    fn can_handle(&self, req: &Message) -> bool {
        matches!(req.body.extra, MessageExtra::Broadcast(_))
    }

    fn handle(&self, node: &Rc<RefCell<Node>>, req: &Message) -> Option<MessageExtra> {
        if let MessageExtra::Broadcast(payload) = &req.body.extra {
            if node.borrow_mut().messages_seen.insert(payload.message) {
                for neibor in node.borrow().topology.get(&node.borrow().id).unwrap() {
                    if neibor == &req.src {
                        continue;
                    }
                    let my_req = Message {
                        src: node.borrow().id.clone(),
                        dest: neibor.clone(),
                        body: MessageBody {
                            msg_id: Some(node.borrow().next_msg_id()),
                            in_reply_to: None,
                            // broadcast message
                            extra: req.body.extra.clone(),
                        },
                    };
                    eprintln!("sent {}", serde_json::to_string(&my_req).unwrap());
                    {
                        let node = node.borrow();
                        let mut unpacked = node.unacked.lock().unwrap();
                        unpacked.insert(
                            my_req.body.msg_id.unwrap(),
                            serde_json::to_string(&my_req).unwrap(),
                        );
                    }
                    node.borrow().send(my_req);
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

    fn handle(&self, node: &Rc<RefCell<Node>>, req: &Message) -> Option<MessageExtra> {
        if let Some(in_reply_to) = &req.body.in_reply_to {
            eprintln!("broadcast ok for message {}", in_reply_to);
            let node = node.borrow();
            node.unacked.lock().unwrap().remove(in_reply_to);
        }
        None
    }
}

#[cfg(not(feature = "lin_kv"))]
pub struct ReadHandler;

#[cfg(not(feature = "lin_kv"))]
impl MessageHandler for ReadHandler {
    fn can_handle(&self, req: &Message) -> bool {
        matches!(req.body.extra, MessageExtra::Read)
    }

    fn handle(&self, node: &Rc<RefCell<Node>>, req: &Message) -> Option<MessageExtra> {
        if let MessageExtra::Read = &req.body.extra {
            Some(MessageExtra::ReadOk(ReadResponseExtra {
                messages: node.borrow().messages_seen.clone(),
            }))
        } else {
            None
        }
    }
}

#[cfg(not(feature = "lin_kv"))]
pub struct ReadOkHandler;

#[cfg(not(feature = "lin_kv"))]
impl MessageHandler for ReadOkHandler {
    fn can_handle(&self, req: &Message) -> bool {
        matches!(req.body.extra, MessageExtra::ReadOk(_))
    }

    fn handle(&self, _node: &Rc<RefCell<Node>>, _req: &Message) -> Option<MessageExtra> {
        None
    }
}

pub struct TxnHandler;

impl MessageHandler for TxnHandler {
    fn can_handle(&self, req: &Message) -> bool {
        matches!(req.body.extra, MessageExtra::Txn(_))
    }

    #[cfg(feature = "lin_kv")]
    fn handle(&self, node: &Rc<RefCell<Node>>, req: &Message) -> Option<MessageExtra> {
        if let MessageExtra::Txn(payload) = &req.body.extra {
            let mut transactor = Transactor::new(node);
            let results = transactor.transact(&payload.txn);

            match results {
                Ok(txn) => {
                    let mytxn = TxnResponseExtra { txn };
                    Some(MessageExtra::TxnOk(mytxn))
                }
                Err(e) => Some(MessageExtra::Error(e)),
            }
        } else {
            None
        }
    }

    #[cfg(not(feature = "lin_kv"))]
    fn handle(&self, node: &Rc<RefCell<Node>>, req: &Message) -> Option<MessageExtra> {
        if let MessageExtra::Txn(payload) = &req.body.extra {
            let mut results = Vec::new();
            for op in &payload.txn {
                if op.0 == "r" {
                    let node = node.borrow();
                    let value = node.kv_store.get(&op.1);
                    results.push(Query(
                        "r".to_string(),
                        op.1,
                        QueryValue::Read(value.cloned()),
                    ));
                }

                if op.0 == "append" {
                    node.borrow_mut()
                        .kv_store
                        .entry(op.1)
                        .or_default()
                        .push(match op.2 {
                            QueryValue::Append(v) => v,
                            QueryValue::Read(_) => panic!("wrong value for 'append' operation"),
                        });
                    results.push(Query("append".to_string(), op.1, op.2.clone()));
                }
            }

            let mytxn = TxnResponseExtra { txn: results };
            Some(MessageExtra::TxnOk(mytxn))
        } else {
            None
        }
    }
}

pub struct TxnOkHandler;

impl MessageHandler for TxnOkHandler {
    fn can_handle(&self, req: &Message) -> bool {
        matches!(req.body.extra, MessageExtra::TxnOk(_))
    }

    fn handle(&self, _node: &Rc<RefCell<Node>>, _req: &Message) -> Option<MessageExtra> {
        None
    }
}
