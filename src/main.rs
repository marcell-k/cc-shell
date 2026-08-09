#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).unwrap() == 0 {
            break;
        }
        let input = input.trim();
        if input.is_empty() {
            continue;
        }
        let mut parts = input.split_whitespace();
        let command = parts.next().unwrap();

        if command == "exit" {
            break;
        } else if command == "echo" {
            let rest: Vec<&str> = parts.collect();
            println!("{}", rest.join(" "));
            continue;
        }
        println!("{}: command not found", command);
    }
}
