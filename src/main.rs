use codecrafters_shell::{
    CommandCompleterHelper, Handler, Io, Job, JobStatus, ParsedCommand, Redirect, RedirectMode,
    build_builtins, jobs_table, next_job_id, parse_command, reap_jobs, search_path, split_pipeline,
    tokenize,
};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Write};
use std::os::unix::process::CommandExt;
use std::process::{Child, Command, Stdio};

use rustyline::error::ReadlineError;
use rustyline::{Config, Editor};

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

            if command.background {
                match cmd.spawn() {
                    Ok(child) => {
                        let pid = child.id();
                        let id = next_job_id();

                        println!("[{}] {}", id, pid);

                        let cmdline = std::iter::once(command.program.clone())
                            .chain(command.args.iter().cloned())
                            .collect::<Vec<_>>()
                            .join(" ");

                        jobs_table().lock().unwrap().push(Job {
                            id,
                            pid,
                            command: cmdline,
                            status: JobStatus::Running,
                            child,
                        });
                    }
                    Err(e) => eprintln!("{}: {}", command.program, e),
                }
            } else {
                if let Err(e) = cmd.status() {
                    eprintln!("{}: {}", command.program, e);
                }
            }
        }
        None => println!("{}: command not found", command.program),
    }
}

fn dispatch_pipeline(mut commands: Vec<ParsedCommand>, builtins: &HashMap<&'static str, Handler>) {
    if commands.len() == 1 {
        let command = commands.pop().unwrap();
        dispatch(command, builtins);
        return;
    }

    let mut children: Vec<Child> = Vec::new();

    let last_index = commands.len() - 1;
    let mut prev_stdio: Option<Stdio> = None;

    for (i, command) in commands.into_iter().enumerate() {
        let is_last = i == last_index;
        if let Some(&handler) = builtins.get(command.program.as_str()) {
            let mut out: Box<dyn Write> = if is_last {
                match &command.stdout_redirect {
                    Some(redirect) => match open_redirect(redirect) {
                        Some(f) => Box::new(f),
                        None => return,
                    },
                    None => Box::new(io::stdout()),
                }
            } else {
                let (reader, writer) = match io::pipe() {
                    Ok(p) => p,
                    Err(e) => {
                        eprintln!("pipe: {}", e);
                        return;
                    }
                };
                prev_stdio = Some(Stdio::from(reader));
                Box::new(writer)
            };

            let mut stderr_handle = io::stderr();
            let mut err: Box<dyn Write> = match &command.stderr_redirect {
                Some(redirect) => match open_redirect(redirect) {
                    Some(f) => Box::new(f),
                    None => return,
                },
                None => Box::new(&mut stderr_handle),
            };

            let mut io_ctx = Io {
                out: &mut *out,
                err: &mut *err,
            };
            handler(&command, &mut io_ctx);
            continue;
        }

        let path = match search_path(&command.program) {
            Some(p) => p,
            None => {
                println!("{}: command not found", command.program);
                return;
            }
        };

        let mut cmd = Command::new(&path);
        cmd.arg0(&command.program).args(&command.args);

        if let Some(stdio) = prev_stdio.take() {
            cmd.stdin(stdio);
        }

        if is_last {
            if let Some(redirect) = &command.stdout_redirect {
                match open_redirect(redirect) {
                    Some(file) => {
                        cmd.stdout(Stdio::from(file));
                    }
                    None => return,
                }
            }
        } else {
            cmd.stdout(Stdio::piped());
        }

        if let Some(redirect) = &command.stderr_redirect {
            match open_redirect(redirect) {
                Some(file) => {
                    cmd.stderr(Stdio::from(file));
                }
                None => return,
            }
        }

        match cmd.spawn() {
            Ok(mut child) => {
                if !is_last && let Some(stdout) = child.stdout.take() {
                    prev_stdio = Some(Stdio::from(stdout));
                }
                children.push(child);
            }
            Err(e) => {
                eprintln!("{}: {}", command.program, e);
                return;
            }
        }
    }

    for mut child in children {
        let _ = child.wait();
    }
}

fn main() -> rustyline::Result<()> {
    let builtins = build_builtins();
    let programs: Vec<&'static str> = builtins.keys().copied().collect();

    let helper = CommandCompleterHelper { programs };
    let config = Config::builder()
        .completion_type(rustyline::CompletionType::List)
        .build();
    let mut rl = Editor::<CommandCompleterHelper, _>::with_config(config)?;
    rl.set_helper(Some(helper));
    loop {
        reap_jobs(&mut io::stdout());
        let input = match rl.readline("$ ") {
            Ok(line) => line,
            Err(ReadlineError::Eof) | Err(ReadlineError::Interrupted) => break,
            Err(e) => return Err(e),
        };
        let input = input.trim();
        if input.is_empty() {
            continue;
        }
        let tokens = tokenize(input);
        if tokens.is_empty() {
            continue;
        }
        let commands: Vec<ParsedCommand> = split_pipeline(tokens)
            .into_iter()
            .map(parse_command)
            .collect();
        dispatch_pipeline(commands, &builtins);
    }
    Ok(())
}
