mod message_handlers;
mod messages;
mod node;

use message_handlers::*;
use messages::*;
use node::*;

use std::io::{self, BufRead};
use std::rc::Rc;
use std::cell::RefCell;

fn main() {
    let stdin = io::stdin().lock();

    let node = Node::new();
    let node = Rc::new(RefCell::new(node));
    node.borrow_mut().start_broadcast_loop();

    let router: Vec<Box<dyn MessageHandler>> = vec![
        Box::new(InitHandler),
        Box::new(InitOkHandler),
        Box::new(EchoHandler),
        Box::new(EchoOkHandler),
        Box::new(GenerateHandler),
        Box::new(GenerateOkHandler),
        Box::new(TopologyHandler),
        Box::new(TopologyOkHandler),
        Box::new(BroadcastHandler),
        Box::new(BroadcastOkHandler),
        #[cfg(not(feature = "lin_kv"))]
        Box::new(ReadHandler),
        #[cfg(not(feature = "lin_kv"))]
        Box::new(ReadOkHandler),
    ];

    for line in stdin.lines() {
        match line {
            Ok(content) => {
                if content.is_empty() {
                    continue;
                }
                match serde_json::from_str::<Message>(&content) {
                    Ok(msg) => {
                        if let Some(response) = Node::handle_message(node.clone(),&msg, &router) {
                            node.borrow_mut().send(response);
                        }
                    }
                    Err(err) => eprintln!("invalid json data for message: {}", err),
                }
            }
            Err(err) => eprintln!("Error reading line: {}", err),
        }
    }
}
