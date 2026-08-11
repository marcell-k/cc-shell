use codecrafters_shell::{Handler, build_builtins, search_path, tokenize, type_cmd};
use std::collections::HashMap;
use std::io::{self, Write};
use std::os::unix::process::CommandExt;
use std::process::Command;

fn dispatch(command: &str, args: &[&str], builtins: &HashMap<&'static str, Handler>) {
    if command == "type" {
        type_cmd(args, builtins);
        return;
    }

    if let Some(handler) = builtins.get(command) {
        handler(args);
        return;
    }

    match search_path(command) {
        Some(path) => {
            let status = Command::new(&path).arg0(command).args(args).status();
            if let Err(e) = status {
                eprintln!("{}: {}", command, e);
            }
        }
        None => println!("{}: command not found", command),
    }
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
        let tokens = tokenize(input);
        if tokens.is_empty() {
            continue;
        }

        let refs: Vec<&str> = tokens.iter().map(String::as_str).collect();
        let (command, args) = refs.split_first().unwrap();
        dispatch(command, args, &builtins);
    }
}
