use codecrafters_shell::{
    Handler, Io, ParsedCommand, Redirect, RedirectMode, build_builtins, parse_command, search_path,
    tokenize,
};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Write};
use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};

fn open_redirect(redirect: &Redirect) -> Option<File> {
    let mut opts = std::fs::OpenOptions::new();
    opts.write(true).create(true);
    match redirect.mode {
        RedirectMode::Truncate => {
            opts.truncate(true);
        }
        RedirectMode::Append => {
            opts.append(true);
        }
    }
    match opts.open(&redirect.path) {
        Ok(f) => Some(f),
        Err(e) => {
            eprintln!("{}: {}", redirect.path.display(), e);
            None
        }
    }
}

fn dispatch(command: ParsedCommand, builtins: &HashMap<&'static str, Handler>) {
    let stdout_file = match &command.stdout_redirect {
        Some(path) => match open_redirect(path) {
            Some(f) => Some(f),
            None => return,
        },
        None => None,
    };
    let stderr_file = match &command.stderr_redirect {
        Some(path) => match open_redirect(path) {
            Some(f) => Some(f),
            None => return,
        },
        None => None,
    };

    if let Some(handler) = builtins.get(command.program.as_str()) {
        let mut stdout_handle = io::stdout();
        let mut stderr_handle = io::stderr();

        let mut out: Box<dyn Write> = stdout_file
            .map_or(Box::new(&mut stdout_handle) as Box<dyn Write>, |f| {
                Box::new(f)
            });
        let mut err: Box<dyn Write> = stderr_file
            .map_or(Box::new(&mut stderr_handle) as Box<dyn Write>, |f| {
                Box::new(f)
            });
        let mut io_ctx = Io {
            out: &mut *out,
            err: &mut *err,
        };
        handler(&command, &mut io_ctx);
        return;
    }

    match search_path(&command.program) {
        Some(path) => {
            let mut cmd = Command::new(&path);
            cmd.arg0(&command.program).args(&command.args);
            if let Some(file) = stdout_file {
                cmd.stdout(Stdio::from(file));
            }
            if let Some(file) = stderr_file {
                cmd.stderr(Stdio::from(file));
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
