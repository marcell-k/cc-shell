use codecrafters_shell::{build_builtins, type_cmd};
use std::io::{self, Write};

fn main() {
    let builtins = build_builtins();
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
        let args: Vec<&str> = parts.collect();

        if command == "type" {
            type_cmd(&args, &builtins);
        } else {
            match builtins.get(command) {
                Some(handler) => handler(&args),
                None => println!("{}: command not found", command),
            }
        }
    }
}
