use std::collections::HashMap;
use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use crate::Token;

pub type Handler = fn(&ParsedCommand);

pub struct ParsedCommand {
    pub program: String,
    pub args: Vec<String>,
    pub stdout_redirect: Option<(u8, PathBuf)>,
}

pub fn parse_command(tokens: Vec<Token>) -> ParsedCommand {
    let mut iter = tokens.into_iter();

    let program = match iter.next() {
        Some(Token::Word(p)) => p,
        _ => String::new(),
    };

    let mut args: Vec<String> = Vec::new();
    let mut stdout_redirect: Option<(u8, PathBuf)> = None;

    while let Some(item) = iter.next() {
        match item {
            Token::Word(w) => args.push(w),
            Token::Redirect => {
                if let Some(Token::Word(w)) = iter.next() {
                    stdout_redirect = Some((1, w.into()))
                }
            }
            Token::Pipe => {}
        }
    }
    ParsedCommand {
        program,
        args,
        stdout_redirect,
    }
}

pub fn exit_cmd(_command: &ParsedCommand) {
    std::process::exit(0)
}

pub fn echo_cmd(command: &ParsedCommand) {
    println!("{}", command.args.join(" "))
}

pub fn pwd_cmd(_command: &ParsedCommand) {
    println!("{}", env::current_dir().unwrap().display())
}

pub fn cd_cmd(command: &ParsedCommand) {
    let target = match command.args.first() {
        Some(t) => t.as_str(),
        None => return,
    };

    let path: PathBuf = if target == "~" {
        match env::var_os("HOME") {
            Some(home) => PathBuf::from(home),
            None => return,
        }
    } else if let Some(rest) = target.strip_prefix("~") {
        match env::var_os("HOME") {
            Some(home) => PathBuf::from(home).join(rest),
            None => return,
        }
    } else {
        PathBuf::from(target)
    };

    if env::set_current_dir(&path).is_err() {
        println!("cd: {}: No such file or directory", target)
    }
}

pub fn type_cmd(command: &ParsedCommand) {
    let target = match command.args.first() {
        Some(t) => t.as_str(),
        None => return,
    };

    let builtins = build_builtins();
    if builtins.contains_key(target) {
        println!("{} is a shell builtin", target);
        return;
    }

    match search_path(target) {
        Some(path) => println!("{} is {}", target, path.display()),
        None => println!("{}: not found", target),
    }
}

pub fn build_builtins() -> HashMap<&'static str, Handler> {
    let mut m: HashMap<&'static str, Handler> = HashMap::new();
    m.insert("exit", exit_cmd);
    m.insert("echo", echo_cmd);
    m.insert("pwd", pwd_cmd);
    m.insert("cd", cd_cmd);
    m.insert("type", type_cmd);
    m
}

pub fn search_path(command: &str) -> Option<PathBuf> {
    let path_var = env::var_os("PATH").unwrap_or_default();

    for dir in env::split_paths(&path_var) {
        let candidate = dir.join(command);
        if let Ok(metadata) = fs::metadata(&candidate) {
            let mode = metadata.permissions().mode();
            if mode & 0o111 != 0 {
                return Some(candidate);
            }
        }
    }
    None
}
