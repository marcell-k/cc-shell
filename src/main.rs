use std::collections::HashMap;
#[allow(unused_imports)]
use std::io::{self, Write};

type Handler = fn(&[&str]);

fn exit_cmd(_args: &[&str]) {
    std::process::exit(0)
}

fn echo_cmd(args: &[&str]) {
    println!("{}", args.join(" "))
}

fn type_cmd(args: &[&str], builtins: &HashMap<&'static str, Handler>) {
    if args.is_empty() {
        return;
    }

    let command = args[0];
    if builtins.contains_key(command) {
        println!("{} is a shell builtin", command)
    } else {
        println!("{}: not found", command)
    }
}

fn build_builtins() -> HashMap<&'static str, Handler> {
    let mut m: HashMap<&'static str, Handler> = HashMap::new();
    m.insert("exit", exit_cmd);
    m.insert("echo", echo_cmd);

    m
}

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
