use std::io::{self, BufRead};

fn main() {
    let stdin = io::stdin().lock();

    for line in stdin.lines() {
        match line {
            Ok(content) => println!("{}", content),
            Err(err) => eprintln!("Error reading line: {}", err),
        }
    }
}
