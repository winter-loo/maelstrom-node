use crate::message_handlers::*;
use crate::messages::*;
use std::collections::HashMap;
use std::collections::HashSet;
use std::rc::Rc;
use std::cell::RefCell;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use std::sync::Mutex;

#[derive(Debug)]
pub struct Node {
    pub id: String,
    pub node_ids: Vec<String>,
    pub topology: HashMap<String, Vec<String>>,
    pub messages_seen: HashSet<BroadcastValue>,
    msg_id: AtomicU64,
    // msg_id -> Message serialized string
    pub unacked: Arc<Mutex<HashMap<u64, String>>>,
    // for txn-list-append challenge
    #[cfg(not(feature = "lin_kv"))]
    pub kv_store: HashMap<usize, Vec<usize>>,
}

impl Node {
    pub fn new() -> Self {
        Node {
            id: String::from(""),
            node_ids: Vec::new(),
            topology: HashMap::new(),
            messages_seen: HashSet::new(),
            msg_id: AtomicU64::new(0),
            unacked: Arc::new(Mutex::new(HashMap::new())),
            #[cfg(not(feature = "lin_kv"))]
            kv_store: HashMap::new(),
        }
    }

    pub fn start_broadcast_loop(&self) {
        let unacked_clone = self.unacked.clone();
        // create a thread to retry unacked messages
        let jh = std::thread::spawn(move || {
            loop {
                // TODO: how long should we wait then retry?
                std::thread::sleep(std::time::Duration::from_millis(200));
                let unacked = unacked_clone.lock().unwrap();
                for (msg_id, msg) in unacked.iter() {
                    eprintln!("retry message {}, {}", msg_id, msg);
                    println!("{}", msg);
                }
            }
        });
        // detach the thread
        drop(jh);
    }

    pub fn next_msg_id(&self) -> u64 {
        self.msg_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }

    pub fn handle_message(
        node: Rc<RefCell<Node>>,
        req: &Message,
        msg_handlers: &[Box<dyn MessageHandler>],
    ) -> Option<Message> {
        let handler = msg_handlers.iter().find(|h| h.can_handle(req));

        match handler {
            Some(handler) => {
                let res_extra = handler.handle(&node, req);
                if res_extra.is_none() {
                    None
                } else {
                    let mut res = Message {
                        src: req.dest.clone(),
                        dest: req.src.clone(),
                        body: MessageBody {
                            msg_id: None,
                            in_reply_to: None,
                            extra: res_extra.unwrap(),
                        },
                    };
                    res.body.in_reply_to = req.body.msg_id;
                    res.body.msg_id = Some(node.borrow().next_msg_id());
                    Some(res)
                }
            }
            None => None,
        }
    }

    pub fn send(&self, res: Message) {
        println!("{}", serde_json::to_string(&res).unwrap());
    }

    #[cfg(feature = "lin_kv")]
    pub fn sync_rpc(&mut self, dest: &str, payload: MessageExtra) -> MessageExtra {
        use std::io::BufRead;
        let req = Message {
            src: self.id.clone(),
            dest: dest.to_string(),
            body: MessageBody {
                msg_id: Some(self.next_msg_id()),
                in_reply_to: None,
                extra: payload,
            },
        };
        println!("{}", serde_json::to_string(&req).unwrap());
        eprintln!("sent to lin-kv: {}", serde_json::to_string(&req).unwrap());
        let mut line = String::new();
        std::io::stdin().lock().read_line(&mut line).unwrap();
        eprintln!("received from lin-kv: {}", line);
        serde_json::from_str::<Message>(&line).unwrap().body.extra
    }
}

// fix clippy issue
impl Default for Node {
    fn default() -> Self {
        Self::new()
    }
}