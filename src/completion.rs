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

        // File name
        if let Some(last_space_idx) = prefix.rfind(" ") {
            let arg_start = last_space_idx + 1;
            let arg_prefix = &prefix[arg_start..];

            let matches: Vec<Pair> = search_filenames(arg_prefix)
                .into_iter()
                .map(|name| Pair {
                    display: name.clone(),
                    replacement: format!("{} ", name),
                })
                .collect();
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

pub fn search_filenames(prefix: &str) -> Vec<String> {
    let mut found: HashSet<String> = HashSet::new();

    let entries = match fs::read_dir(".") {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        let name = entry.file_name().to_string_lossy().into_owned();

        if name.starts_with(prefix) {
            found.insert(name);
        }
    }

    found.into_iter().collect()
}
