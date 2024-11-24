mod message_handlers;
mod messages;
mod node;

use message_handlers::*;
use messages::*;
use node::*;

use std::io::{self, BufRead};

fn main() {
    let stdin = io::stdin().lock();

    let mut node = Node::new();

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
        Box::new(ReadHandler),
        Box::new(ReadOkHandler),
    ];

    for line in stdin.lines() {
        match line {
            Ok(content) => {
                if content.is_empty() {
                    continue;
                }
                match serde_json::from_str::<Message>(& content) {
                    Ok(msg) => {
                        if let Some(response) = node.handle_message(&msg, &router) {
                            node.send(response);
                        }
                    }
                    Err(err) => eprintln!("invalid json data for message: {}", err),
                }
            }
            Err(err) => eprintln!("Error reading line: {}", err),
        }
    }
}
