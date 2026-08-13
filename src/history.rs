use std::{
    io,
    sync::{Mutex, OnceLock},
};
pub static HISTORY: OnceLock<Mutex<Vec<String>>> = OnceLock::new();

pub fn history() -> &'static Mutex<Vec<String>> {
    HISTORY.get_or_init(|| Mutex::new(Vec::new()))
}

pub fn add_history(line: &str) {
    history().lock().unwrap().push(line.to_string());
}

pub fn print_history(out: &mut dyn io::Write, limit: Option<usize>) {
    let entries = history().lock().unwrap();
    let start = match limit {
        Some(n) => entries.len().saturating_sub(n),
        None => 0,
    };
    for (i, entry) in entries[start..].iter().enumerate() {
        writeln!(out, "{:>4}  {}", start + i + 1, entry).ok();
    }
}
