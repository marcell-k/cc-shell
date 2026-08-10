use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::{collections::HashMap, env};

pub type Handler = fn(&[&str]);

pub fn exit_cmd(_args: &[&str]) {
    std::process::exit(0)
}

pub fn echo_cmd(args: &[&str]) {
    println!("{}", args.join(" "))
}

pub fn pwd_cmd(_args: &[&str]) {
    println!("{}", env::current_dir().unwrap().display())
}

pub fn cd_cmd(args: &[&str]) {
    if args.is_empty() {
        return;
    }
    let target = args[0];

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

pub fn type_cmd(args: &[&str], builtins: &HashMap<&'static str, Handler>) {
    if args.is_empty() {
        return;
    }
    let command = args[0];

    if builtins.contains_key(command) {
        println!("{} is a shell builtin", command);
        return;
    }

    match search_path(command) {
        Some(path) => println!("{} is {}", command, path.display()),
        None => println!("{}: not found", command),
    }
}

pub fn build_builtins() -> HashMap<&'static str, Handler> {
    let mut m: HashMap<&'static str, Handler> = HashMap::new();
    m.insert("exit", exit_cmd);
    m.insert("echo", echo_cmd);
    m.insert("pwd", pwd_cmd);
    m.insert("cd", cd_cmd);
    m.insert("type", |_args: &[&str]| {});
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
