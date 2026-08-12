use std::process::Child;
use std::sync::{Mutex, OnceLock};

pub struct Job {
    pub id: u32,
    pub pid: u32,
    pub command: String,
    pub child: Child,
}
pub static JOBS: OnceLock<Mutex<Vec<Job>>> = OnceLock::new();

pub fn jobs_table() -> &'static Mutex<Vec<Job>> {
    JOBS.get_or_init(|| Mutex::new(Vec::new()))
}

pub fn next_job_id() -> u32 {
    jobs_table().lock().unwrap().len() as u32 + 1
}
