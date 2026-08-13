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

pub fn print_history(out: &mut dyn io::Write) {
    let entries = history().lock().unwrap();
    for (i, entry) in entries.iter().enumerate() {
        writeln!(out, "{:>4} {}", i + 1, entry).ok();
    }
}
