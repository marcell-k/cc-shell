use std::io::Write;
use std::process::Child;
use std::sync::{Mutex, OnceLock};

pub struct Job {
    pub id: u32,
    pub pid: u32,
    pub command: String,
    pub status: JobStatus,
    pub child: Child,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum JobStatus {
    Running,
    Done,
    // Stopped,
}
pub static JOBS: OnceLock<Mutex<Vec<Job>>> = OnceLock::new();

pub fn jobs_table() -> &'static Mutex<Vec<Job>> {
    JOBS.get_or_init(|| Mutex::new(Vec::new()))
}

pub fn next_job_id() -> u32 {
    jobs_table().lock().unwrap().len() as u32 + 1
}

pub fn reap_jobs(io: &mut dyn Write) {
    let mut jobs = jobs_table().lock().unwrap();

    for job in jobs.iter_mut() {
        if job.status == JobStatus::Running
            && let Ok(Some(_exit_status)) = job.child.try_wait()
        {
            job.status = JobStatus::Done;
        }
    }

    let mut ids: Vec<u32> = jobs.iter().map(|j| j.id).collect();
    ids.sort_unstable_by(|a, b| b.cmp(a));
    let current = ids.first().copied();
    let previous = ids.get(1).copied();

    let mut done: Vec<&Job> = jobs
        .iter()
        .filter(|j| j.status == JobStatus::Done)
        .collect();
    done.sort_unstable_by_key(|j| j.id);

    for job in done {
        let marker = if Some(job.id) == current {
            '+'
        } else if Some(job.id) == previous {
            '-'
        } else {
            ' '
        };
        writeln!(io, "[{}]{}  {:<24}{}", job.id, marker, "Done", job.command).ok();
    }

    jobs.retain(|job| job.status != JobStatus::Done);
}
