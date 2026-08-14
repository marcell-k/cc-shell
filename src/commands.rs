use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use crate::history;
use crate::{RedirectMode, Token, completions, jobs_cmd};

pub type Handler = fn(&ParsedCommand, &mut Io);

pub struct Io<'a> {
    pub out: &'a mut dyn Write,
    pub err: &'a mut dyn Write,
}

pub struct Redirect {
    pub path: PathBuf,
    pub mode: RedirectMode,
}
pub struct ParsedCommand {
    pub program: String,
    pub args: Vec<String>,
    pub stdout_redirect: Option<Redirect>,
    pub stderr_redirect: Option<Redirect>,
    pub background: bool,
}

pub fn parse_command(tokens: Vec<Token>) -> ParsedCommand {
    let mut iter = tokens.into_iter();
    let program = match iter.next() {
        Some(Token::Word(p)) => p,
        _ => String::new(),
    };
    let mut args: Vec<String> = Vec::new();
    let mut stdout_redirect: Option<Redirect> = None;
    let mut stderr_redirect: Option<Redirect> = None;
    let mut background = false;
    while let Some(item) = iter.next() {
        match item {
            Token::Word(w) => args.push(w),
            Token::Redirect(fd, mode) => {
                if let Some(Token::Word(w)) = iter.next() {
                    match fd {
                        2 => {
                            stderr_redirect = Some(Redirect {
                                path: w.into(),
                                mode,
                            });
                        }
                        _ => {
                            stdout_redirect = Some(Redirect {
                                path: w.into(),
                                mode,
                            });
                        }
                    }
                }
            }
            Token::Pipeline => {}
            Token::Background => background = true,
        }
    }
    ParsedCommand {
        program,
        args,
        stdout_redirect,
        stderr_redirect,
        background,
    }
}

pub fn split_pipeline(tokens: Vec<Token>) -> Vec<Vec<Token>> {
    let mut groups: Vec<Vec<Token>> = vec![Vec::new()];
    for token in tokens {
        match token {
            Token::Pipeline => groups.push(Vec::new()),
            other => groups.last_mut().unwrap().push(other),
        }
    }
    groups
}

pub fn complete_cmd(command: &ParsedCommand, io: &mut Io) {
    match command.args.first().map(String::as_str) {
        Some("-p") => {
            if let Some(name) = command.args.get(1) {
                match completions().lock().unwrap().get(name) {
                    Some(path) => {
                        writeln!(io.out, "complete -C '{}' {}", path, name).ok();
                    }
                    None => {
                        writeln!(io.err, "complete: {}: no completion specification", name).ok();
                    }
                }
            }
        }
        Some("-C") => {
            if let (Some(path), Some(name)) = (command.args.get(1), command.args.get(2)) {
                completions()
                    .lock()
                    .unwrap()
                    .insert(name.clone(), path.clone());
            }
        }
        Some("-r") => {
            if let Some(name) = command.args.get(1) {
                completions().lock().unwrap().remove(name);
            }
        }
        _ => {}
    }
}

pub fn history_cmd(command: &ParsedCommand, io: &mut Io) {
    match command.args.first().map(String::as_str) {
        Some("-a") => {
            if let Some(path) = command.args.get(1)
                && let Err(e) = crate::history::append_history_to_file(path)
            {
                writeln!(io.err, "history: {}: {}", path, e).ok();
            }
        }
        Some("-r") => {
            if let Some(path) = command.args.get(1)
                && let Err(e) = history::load_history_from_file(path)
            {
                writeln!(io.err, "history: {} {}", path, e).ok();
            }
        }
        Some("-w") => {
            if let Some(path) = command.args.get(1)
                && let Err(e) = crate::history::print_history_to_file(path)
            {
                writeln!(io.err, "history: {}: {}", path, e).ok();
            }
        }
        _first_arg => {
            let limit = command.args.first().and_then(|s| s.parse::<usize>().ok());
            if let Err(e) = crate::history::print_history(io.out, limit) {
                writeln!(io.err, "history: {}", e).ok();
            }
        }
    }
}

pub fn exit_cmd(_command: &ParsedCommand, _io: &mut Io) {
    std::process::exit(0)
}

pub fn echo_cmd(command: &ParsedCommand, io: &mut Io) {
    writeln!(io.out, "{}", command.args.join(" ")).ok();
}

pub fn pwd_cmd(_command: &ParsedCommand, io: &mut Io) {
    writeln!(io.out, "{}", env::current_dir().unwrap().display()).ok();
}

pub fn cd_cmd(command: &ParsedCommand, io: &mut Io) {
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
        writeln!(io.err, "cd: {}: No such file or directory", target).ok();
    }
}

pub fn type_cmd(command: &ParsedCommand, io: &mut Io) {
    let target = match command.args.first() {
        Some(t) => t.as_str(),
        None => return,
    };
    let builtins = build_builtins();
    if builtins.contains_key(target) {
        writeln!(io.out, "{} is a shell builtin", target).ok();
        return;
    }
    match search_path(target) {
        Some(path) => {
            writeln!(io.out, "{} is {}", target, path.display()).ok();
        }
        None => {
            writeln!(io.out, "{}: not found", target).ok();
        }
    }
}

pub fn build_builtins() -> HashMap<&'static str, Handler> {
    let mut m: HashMap<&'static str, Handler> = HashMap::new();
    m.insert("exit", exit_cmd);
    m.insert("echo", echo_cmd);
    m.insert("pwd", pwd_cmd);
    m.insert("cd", cd_cmd);
    m.insert("type", type_cmd);
    m.insert("complete", complete_cmd);
    m.insert("jobs", jobs_cmd);
    m.insert("history", history_cmd);
    m
}

pub fn search_path(program: &str) -> Option<PathBuf> {
    let path_var = env::var_os("PATH").unwrap_or_default();
    for dir in env::split_paths(&path_var) {
        let candidate = dir.join(program);
        if let Ok(metadata) = fs::metadata(&candidate) {
            let mode = metadata.permissions().mode();
            if mode & 0o111 != 0 {
                return Some(candidate);
            }
        }
    }
    None
}
