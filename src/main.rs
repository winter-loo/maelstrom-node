use serde::{Serialize, Deserialize};
use serde_json::Value;
use std::{io::{self, BufRead}, collections::HashMap};

/// protocol specification from https://github.com/jepsen-io/maelstrom/blob/main/doc/protocol.md
#[derive(Serialize, Deserialize, Debug, Clone)]
struct Message {
    src: String,
    dest: String,
    body: MessageBody,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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

    let mut json_buf = String::from("");
    for line in stdin.lines() {
        match line {
            Ok(content) => {
                if content.is_empty()  && !json_buf.is_empty() {
                    match serde_json::from_str::<Message>(&json_buf) {
                        Ok(msg) => println!("{}", serde_json::to_string(&handle_message(msg)).unwrap()),
                        Err(err) => eprintln!("invalid json data for message: {}", err),
                    }
                    json_buf.clear();
                } else if !content.is_empty() {
                    json_buf += &content;
                }
            }
            Err(err) => eprintln!("Error reading line: {}", err),
        }
    }
}

fn handle_message(msg: Message) -> Message {
    if msg.body.type_ == "echo" {
        let mut res = msg.clone();
        let req_id = msg.body.msg_id.unwrap();
        res.src = msg.dest;
        res.dest = msg.src;
        res.body.type_ = "echo_ok".to_string();
        res.body.msg_id = Some(req_id + 1);
        res.body.in_reply_to = Some(req_id);
        return res;
    }
    panic!("unknown message");
}
