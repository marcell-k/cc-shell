use codecrafters_shell::{
    Handler, ParsedCommand, build_builtins, parse_command, search_path, tokenize,
};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Write};
use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};

fn dispatch(command: ParsedCommand, builtins: &HashMap<&'static str, Handler>) {
    let stdout_file = match command.stdout_redirect.as_ref() {
        Some((_fd, path)) => match File::create(path) {
            Ok(f) => Some(f),
            Err(e) => {
                eprintln!("{}: {}", path.display(), e);
                return;
            }
        },
        None => None,
    };

    if let Some(handler) = builtins.get(command.program.as_str()) {
        match stdout_file {
            Some(mut f) => handler(&command, &mut f),
            None => {
                let mut stdout = io::stdout();
                handler(&command, &mut stdout);
            }
        }
        return;
    }

    match search_path(&command.program) {
        Some(path) => {
            let mut cmd = Command::new(&path);
            cmd.arg0(&command.program).args(&command.args);
            if let Some(file) = stdout_file {
                cmd.stdout(Stdio::from(file));
            }
            if let Err(e) = cmd.status() {
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
