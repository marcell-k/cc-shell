use std::{
    fs::{File, OpenOptions},
    io::{self, BufRead, BufReader, BufWriter, Write},
    path::Path,
    sync::{Mutex, OnceLock},
};

#[derive(Default, Debug)]
struct History {
    entries: Vec<String>,
    append_cursor: usize,
}

impl History {
    pub fn add(&mut self, line: impl Into<String>) {
        self.entries.push(line.into());
    }

    pub fn print(&self, out: &mut dyn Write, limit: Option<usize>) -> io::Result<()> {
        let start = match limit {
            Some(n) => self.entries.len().saturating_sub(n),
            None => 0,
        };

        for (i, entry) in self.entries[start..].iter().enumerate() {
            writeln!(out, "{:>4}  {}", start + i + 1, entry)?;
        }
        Ok(())
    }

    pub fn load_from_file(&mut self, path: impl AsRef<Path>) -> io::Result<()> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if !line.is_empty() {
                self.add(line);
            }
        }
        Ok(())
    }

    pub fn write_to_file(&mut self, path: impl AsRef<Path>) -> io::Result<()> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        for entry in &self.entries {
            writeln!(writer, "{entry}")?;
        }
        writer.flush()?;
        self.append_cursor = self.entries.len();
        Ok(())
    }

    pub fn append_to_file(&mut self, path: impl AsRef<Path>) -> io::Result<()> {
        if self.append_cursor >= self.entries.len() {
            return Ok(());
        }

        let file = OpenOptions::new().append(true).create(true).open(path)?;
        let mut writer = BufWriter::new(file);

        for entry in &self.entries[self.append_cursor..] {
            writeln!(writer, "{entry}")?;
        }

        writer.flush()?;
        self.append_cursor = self.entries.len();
        Ok(())
    }
}

static HISTORY: OnceLock<Mutex<History>> = OnceLock::new();

fn history() -> &'static Mutex<History> {
    HISTORY.get_or_init(|| Mutex::new(History::default()))
}

pub fn add_history(line: impl Into<String>) {
    let mut guard = history().lock().unwrap_or_else(|e| e.into_inner());
    guard.add(line);
}

pub fn print_history(out: &mut dyn Write, limit: Option<usize>) -> io::Result<()> {
    let guard = history().lock().unwrap_or_else(|e| e.into_inner());
    guard.print(out, limit)
}

pub fn load_history_from_file(path: impl AsRef<Path>) -> io::Result<()> {
    let mut guard = history().lock().unwrap_or_else(|e| e.into_inner());
    guard.load_from_file(path)
}

pub fn print_history_to_file(path: impl AsRef<Path>) -> io::Result<()> {
    let mut guard = history().lock().unwrap_or_else(|e| e.into_inner());
    guard.write_to_file(path)
}

pub fn append_history_to_file(path: impl AsRef<Path>) -> io::Result<()> {
    let mut guard = history().lock().unwrap_or_else(|e| e.into_inner());
    guard.append_to_file(path)
}
