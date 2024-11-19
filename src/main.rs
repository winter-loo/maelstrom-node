use serde::{Serialize, Deserialize};
use serde_json::Value;
use std::{io::{self, BufRead}, collections::HashMap};

/// protocol specification from https://github.com/jepsen-io/maelstrom/blob/main/doc/protocol.md
#[derive(Serialize, Deserialize, Debug)]
struct Message {
    src: String,
    dest: String,
    body: MessageBody,
}

#[derive(Serialize, Deserialize, Debug)]
struct MessageBody {
    // reference: https://serde.rs/field-attrs.html
    #[serde(rename = "type")]
    type_: String,
    msg_id: Option<u64>,
    in_reply_to: Option<u64>,
    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

fn main() {
    let stdin = io::stdin().lock();

    for line in stdin.lines() {
        match line {
            Ok(content) => match serde_json::from_str::<Message>(&content) {
                Ok(msg) => println!("{:#?}", msg),
                Err(err) => eprintln!("invalid json data for message: {}", err),
            },
            Err(err) => eprintln!("Error reading line: {}", err),
        }
    }
}
