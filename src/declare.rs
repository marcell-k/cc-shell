use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
};
pub static VARIABLES: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();

pub fn variables() -> &'static Mutex<HashMap<String, String>> {
    VARIABLES.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn set_var(key: impl Into<String>, val: impl Into<String>) {
    let mut vars = variables().lock().unwrap();
    vars.insert(key.into(), val.into());
}

pub fn get_var(key: &str) -> Option<String> {
    let vars = variables().lock().unwrap();
    vars.get(key).cloned()
}

pub fn is_valid_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}
