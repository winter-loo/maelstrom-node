use maelstrom_node::messages::*;
use maelstrom_node::message_handlers::*;
use maelstrom_node::node::*;

use std::io::{self, BufRead};

fn main() {
    let stdin = io::stdin().lock();

    let mut node = Node::new();

    let router: Vec<Box<dyn MessageHandler>> = vec![
        Box::new(InitHandler),
        Box::new(InitOkHandler),
        Box::new(TxnHandler),
        Box::new(TxnOkHandler),
    ];

    for line in stdin.lines() {
        match line {
            Ok(content) => {
                if content.is_empty() {
                    continue;
                }
                match serde_json::from_str::<Message>(&content) {
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
