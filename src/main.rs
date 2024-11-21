mod messages;
mod node;

use messages::*;
use node::*;

use std::io::{self, BufRead};

fn main() {
    let stdin = io::stdin().lock();

    let mut node = Node::new();

    for line in stdin.lines() {
        match line {
            Ok(content) => {
                if content.is_empty() {
                    continue;
                }
                match serde_json::from_str::<Message>(&content) {
                    Ok(msg) => {
                        let response = node.handle_message(msg);
                        node.send(response);
                    }
                    Err(err) => eprintln!("invalid json data for message: {}", err),
                }
            }
            Err(err) => eprintln!("Error reading line: {}", err),
        }
    }
}
