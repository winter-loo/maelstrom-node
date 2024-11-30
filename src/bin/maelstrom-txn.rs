use maelstrom_node::messages::*;
use maelstrom_node::message_handlers::*;
use maelstrom_node::node::*;

use std::io::{self, BufRead};

fn main() {
    let mut node = Node::new();

    let router: Vec<Box<dyn MessageHandler>> = vec![
        Box::new(InitHandler),
        Box::new(InitOkHandler),
        Box::new(TxnHandler),
        Box::new(TxnOkHandler),
    ];

    loop {
        let line = {
            // unlock as soon as possible
            let mut stdin = io::stdin().lock();
            let mut buffer = String::new();
            match stdin.read_line(&mut buffer) {
                Ok(0) => break, // EOF reached, exit the loop
                Ok(_) => buffer,
                Err(e) => panic!("Error reading second line: {e}"),
            }
        };

        match serde_json::from_str::<Message>(&line) {
            Ok(msg) => {
                if let Some(response) = node.handle_message(&msg, &router) {
                    node.send(response);
                }
            }
            Err(err) => eprintln!("invalid json data for message: {}", err),
        }
    }
}
