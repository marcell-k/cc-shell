use super::{Io, ParsedCommand};
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

    poll_jobs(&mut jobs);
    let (current, previous) = job_markers(&jobs);

    let mut done: Vec<&Job> = jobs
        .iter()
        .filter(|j| j.status == JobStatus::Done)
        .collect();
    done.sort_unstable_by_key(|j| j.id);

    for job in done {
        let marker = marker_for(job.id, current, previous);
        writeln!(io, "{}", format_job_line(job, marker)).ok();
    }

    jobs.retain(|job| job.status != JobStatus::Done);
}

fn poll_jobs(jobs: &mut [Job]) {
    for job in jobs.iter_mut() {
        if job.status == JobStatus::Running
            && let Ok(Some(_exit_status)) = job.child.try_wait()
        {
            job.status = JobStatus::Done;
        }
    }
}

fn job_markers(jobs: &[Job]) -> (Option<u32>, Option<u32>) {
    let mut ids: Vec<u32> = jobs.iter().map(|j| j.id).collect();
    ids.sort_unstable_by(|a, b| b.cmp(a)); // descending: highest id first
    (ids.first().copied(), ids.get(1).copied())
}

fn marker_for(id: u32, current: Option<u32>, previous: Option<u32>) -> char {
    if Some(id) == current {
        '+'
    } else if Some(id) == previous {
        '-'
    } else {
        ' '
    }
}

fn format_job_line(job: &Job, marker: char) -> String {
    match job.status {
        JobStatus::Done => format!("[{}]{}  {:<24}{}", job.id, marker, "Done", job.command),
        JobStatus::Running => format!("[{}]{}  {:<24}{} &", job.id, marker, "Running", job.command),
    }
}

pub fn jobs_cmd(_command: &ParsedCommand, io: &mut Io) {
    let mut jobs = jobs_table().lock().unwrap();
    poll_jobs(&mut jobs);
    let (current, previous) = job_markers(&jobs);

    let mut display: Vec<&Job> = jobs.iter().collect();
    display.sort_unstable_by_key(|j| j.id);

    for job in display {
        let marker = marker_for(job.id, current, previous);
        writeln!(io.out, "{}", format_job_line(job, marker)).ok();
    }

    jobs.retain(|job| job.status != JobStatus::Done);
}
