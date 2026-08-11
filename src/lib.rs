use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::{collections::HashMap, env};
use std::{fs, mem};

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
#[derive(PartialEq)]
enum QuoteState {
    None,
    Single,
    Double,
}
pub fn tokenize(input: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut buf = String::new();
    let mut quote = QuoteState::None;
    let mut in_token = false;

    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\\' if quote == QuoteState::None => {
                if let Some(next) = chars.next() {
                    buf.push(next);
                }
                in_token = true;
            }

            '\'' if quote != QuoteState::Double => {
                quote = if quote == QuoteState::Single {
                    QuoteState::None
                } else {
                    QuoteState::Single
                };
                in_token = true;
            }

            '\"' if quote != QuoteState::Single => {
                quote = if quote == QuoteState::Double {
                    QuoteState::None
                } else {
                    QuoteState::Double
                };
                in_token = true;
            }
            ' ' | '\t' if quote == QuoteState::None => {
                if in_token {
                    out.push(mem::take(&mut buf));
                    in_token = false;
                }
            }
            _ => {
                buf.push(c);
                in_token = true;
            }
        }
    }
    if in_token {
        out.push(buf);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::tokenize;

    #[test]
    fn test_single_quote_tokenize() {
        assert_eq!(tokenize("hello"), vec!["hello"]);
        assert_eq!(tokenize("hello world"), vec!["hello", "world"]);
        assert_eq!(tokenize("hello    world"), vec!["hello", "world"]);
        assert_eq!(tokenize("'hello    world'"), vec!["hello    world"]);
        assert_eq!(tokenize("''"), vec![""]);
        assert_eq!(tokenize("hello''world"), vec!["helloworld"]);
        assert_eq!(tokenize("'hello''world'"), vec!["helloworld"]);
        assert_eq!(
            tokenize("cat '/tmp/file name' '/tmp/file name with spaces'"),
            vec!["cat", "/tmp/file name", "/tmp/file name with spaces"]
        );
    }
    #[test]
    fn test_double_quote_tokenize() {
        assert_eq!(
            tokenize("echo \"hello    world\""),
            vec!["echo", "hello    world"]
        ); // spaces preserved inside double quotes

        assert_eq!(
            tokenize("echo \"hello\"\"world\""),
            vec!["echo", "helloworld"]
        ); // adjacent double-quoted strings concatenate

        assert_eq!(tokenize("echo \"hello\"world"), vec!["echo", "helloworld"]); // quoted + unquoted concatenate

        assert_eq!(
            tokenize("echo \"hello\" \"world\""),
            vec!["echo", "hello", "world"]
        ); // space outside quotes = separate args

        assert_eq!(
            tokenize("echo \"shell's test\""),
            vec!["echo", "shell's test"]
        ); // single quote literal inside double quotes

        assert_eq!(
            tokenize("echo 'hello'\"world\""),
            vec!["echo", "helloworld"]
        ); // single + double quote concat, mixed

        assert_eq!(tokenize("echo \"\""), vec!["echo", ""]); // empty double-quoted arg
    }
}
