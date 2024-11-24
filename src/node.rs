use crate::message_handlers::*;
use crate::messages::*;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use std::sync::Mutex;

pub struct Node {
    pub id: String,
    pub node_ids: Vec<String>,
    pub topology: HashMap<String, Vec<String>>,
    pub messages_seen: HashSet<BroadcastValue>,
    msg_id: AtomicU64,
    // msg_id -> Message serialized string
    pub unacked: Arc<Mutex<HashMap<u64, String>>>,
}

impl Node {
    pub fn new() -> Self {
        let unacked = Arc::new(Mutex::new(HashMap::new()));
        let unacked_clone = unacked.clone();
        // create a thread to retry unacked messages
        std::thread::spawn(move || {
            loop {
                // TODO: how long should we wait then retry?
                std::thread::sleep(std::time::Duration::from_millis(5000));
                let unacked = unacked_clone.lock().unwrap();
                for (msg_id, msg) in unacked.iter() {
                    eprintln!("retry message {}, {}", msg_id, msg);
                    println!("{}", msg);
                }
            }
        });

        Node {
            id: String::from(""),
            node_ids: Vec::new(),
            topology: HashMap::new(),
            messages_seen: HashSet::new(),
            msg_id: AtomicU64::new(0),
            unacked,
        }
    }

    pub fn next_msg_id(&self) -> u64 {
        self.msg_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }

    pub fn handle_message(
        &mut self,
        req: &Message,
        msg_handlers: &[Box<dyn MessageHandler>],
    ) -> Option<Message> {
        let handler = msg_handlers.iter().find(|h| h.can_handle(req));

        match handler {
            Some(handler) => {
                let res_extra = handler.handle(self, req);
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
                    res.body.msg_id = Some(self.next_msg_id());
                    Some(res)
                }
            }
            None => {
                eprintln!("unknow message: {:#?}", req);
                None
            }
        }
    }

    pub fn send(&self, res: Message) {
        println!("{}", serde_json::to_string(&res).unwrap());
    }
}
