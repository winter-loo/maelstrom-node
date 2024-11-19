use serde_json::Value;
use std::io::{self, BufRead};

fn main() {
    let stdin = io::stdin().lock();

    for line in stdin.lines() {
        match line {
            Ok(content) => match serde_json::from_str::<Value>(&content) {
                Ok(json_data) => println!("src: {}, dest: {}", json_data["src"], json_data["dest"]),
                Err(err) => eprintln!("invalid json data: {}", err),
            },
            Err(err) => eprintln!("Error reading line: {}", err),
        }
    }
}
