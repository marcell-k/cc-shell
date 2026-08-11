use codecrafters_shell::{
    Handler, ParsedCommand, build_builtins, parse_command, search_path, tokenize,
};
use std::collections::HashMap;
use std::io::{self, Write};
use std::os::unix::process::CommandExt;
use std::process::Command;

fn dispatch(command: ParsedCommand, builtins: &HashMap<&'static str, Handler>) {
    if let Some(handler) = builtins.get(command.program.as_str()) {
        handler(&command);
        return;
    }

    match search_path(&command.program) {
        Some(path) => {
            let status = Command::new(&path)
                .arg0(&command.program)
                .args(&command.args)
                .status();
            if let Err(e) = status {
                eprintln!("{}: {}", command.program, e);
            }
        }
        None => println!("{}: command not found", command.program),
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

        let command = parse_command(tokens);
        dispatch(command, &builtins);
    }
}
