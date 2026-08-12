use rustyline::{
    Context, Helper,
    completion::{Completer, Pair},
    highlight::Highlighter,
    hint::Hinter,
    validate::Validator,
};
use std::os::unix::fs::PermissionsExt;
use std::{env, fs};

use std::collections::HashSet;
use std::result::Result;

use std::collections::HashMap;
use std::process::Command;
use std::sync::Mutex;
use std::sync::OnceLock;

pub static COMPLETIONS: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();

pub fn completions() -> &'static Mutex<HashMap<String, String>> {
    COMPLETIONS.get_or_init(|| Mutex::new(HashMap::new()))
}

pub struct CommandCompleterHelper {
    pub programs: Vec<&'static str>,
}

impl Completer for CommandCompleterHelper {
    type Candidate = Pair;
    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Self::Candidate>), rustyline::error::ReadlineError> {
        let prefix = &line[..pos];
        if line.is_empty() {
            return Ok((0, Vec::new()));
        }
        // command name is always the first whitespace-separated word
        let cmd_name = line.split_whitespace().next().unwrap_or("");

        if let Some(script_path) = completions().lock().unwrap().get(cmd_name).cloned() {
            let arg_start = prefix.rfind(' ').map(|i| i + 1).unwrap_or(prefix.len());
            let word_being_completed = &prefix[arg_start..];

            let before = prefix[..arg_start].trim_end();
            let prev_word = before.split_whitespace().last().unwrap_or("");

            let output = match Command::new(&script_path)
                .arg(cmd_name)
                .arg(word_being_completed)
                .arg(prev_word)
                .env("COMP_LINE", line)
                .env("COMP_POINT", pos.to_string())
                .output()
            {
                Ok(o) => o,
                Err(_) => return Ok((arg_start, Vec::new())),
            };

            let stdout = String::from_utf8_lossy(&output.stdout);
            let mut matches: Vec<Pair> = stdout
                .lines()
                .map(|canditate| Pair {
                    display: canditate.to_string(),
                    replacement: format!("{} ", canditate),
                })
                .collect();
            matches.sort_by(|a, b| a.display.cmp(&b.display));

            return Ok((arg_start, matches));
        }

        // File name
        if let Some(last_space_idx) = prefix.rfind(" ") {
            let arg_start = last_space_idx + 1;
            let arg_prefix = &prefix[arg_start..];

            let mut matches: Vec<Pair> = search_filenames(arg_prefix)
                .into_iter()
                .map(|(name, is_dir)| Pair {
                    display: name.clone(),
                    replacement: if is_dir { name } else { format!("{} ", name) },
                })
                .collect();
            matches.sort_by(|a, b| a.display.cmp(&b.display));
            return Ok((arg_start, matches));
        }

        let builtin_matches = self
            .programs
            .iter()
            .filter(|p| p.starts_with(prefix))
            .map(|cmd| Pair {
                display: cmd.to_string(),
                replacement: format!("{} ", cmd),
            });

        let exec_matches = search_executables(prefix).into_iter().map(|name| Pair {
            display: name.clone(),
            replacement: format!("{} ", name),
        });

        let mut matches: Vec<Pair> = builtin_matches.chain(exec_matches).collect();
        matches.sort_by(|a, b| a.display.cmp(&b.display));
        Ok((0, matches))
    }
}

impl Hinter for CommandCompleterHelper {
    type Hint = String;
}
impl Highlighter for CommandCompleterHelper {}
impl Validator for CommandCompleterHelper {}
impl Helper for CommandCompleterHelper {}

pub fn search_executables(prefix: &str) -> Vec<String> {
    let path_var = env::var_os("PATH").unwrap_or_default();
    let mut found: HashSet<String> = HashSet::new();

    for dir in env::split_paths(&path_var) {
        let entries = match fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue,
        };

        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            let name = entry.file_name().to_string_lossy().into_owned();

            if !name.starts_with(prefix) {
                continue;
            }

            if let Ok(metadata) = entry.metadata()
                && metadata.is_file()
                && metadata.permissions().mode() & 0o111 != 0
            {
                found.insert(name);
            }
        }
    }
    found.into_iter().collect()
}

pub fn search_filenames(prefix: &str) -> Vec<(String, bool)> {
    let mut found: HashSet<(String, bool)> = HashSet::new();

    let (dir_path, name_prefix) = match prefix.rfind('/') {
        Some(idx) => (&prefix[..=idx], &prefix[idx + 1..]),
        None => ("", prefix),
    };

    let read_dir_target = if dir_path.is_empty() { "." } else { dir_path };

    let entries = match fs::read_dir(read_dir_target) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        let name = entry.file_name().to_string_lossy().into_owned();

        if name.starts_with(name_prefix) {
            let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
            let full = if is_dir {
                format!("{}{}/", dir_path, name) // append slash now, no space added later
            } else {
                format!("{}{}", dir_path, name)
            };
            found.insert((full, is_dir));
        }
    }

    found.into_iter().collect()
}
