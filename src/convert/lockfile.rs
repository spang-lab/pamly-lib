use anyhow::{bail, Result};

use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct LockFile {
    #[serde(skip)]
    path: Option<PathBuf>,
    pub state: String,
    pub total: u64,
    pub current: u64,
    pub error: Option<String>,
}

fn get_lock_path(path: &PathBuf) -> Result<PathBuf> {
    let filename = "pamly.lock";
    let base_path = if path.is_dir() {
        path.clone()
    } else {
        match path.parent() {
            Some(p) => p.to_owned(),
            None => bail!("Could not get path parent"),
        }
    };
    let lock_path = base_path.join(filename);
    Ok(lock_path)
}

impl LockFile {
    pub fn exists(path: &PathBuf) -> Result<bool> {
        let lock_path = get_lock_path(path)?;
        Ok(lock_path.is_file())
    }

    pub fn find(path: &PathBuf) -> Result<Option<LockFile>> {
        let lock_path = get_lock_path(path)?;
        if !lock_path.is_file() {
            return Ok(None);
        }
        let file = File::open(&lock_path)?;
        let mut lock: LockFile = serde_json::from_reader(file)?;
        lock.path = Some(lock_path);
        Ok(Some(lock))
    }
    pub fn lock(path: &PathBuf, state: &str) -> Result<LockFile> {
        let lock_path = get_lock_path(path)?;
        let lock = LockFile {
            path: Some(lock_path),
            state: state.to_owned(),
            total: 0,
            current: 0,
            error: None,
        };
        lock.write()?;
        Ok(lock)
    }
    pub fn release(&self) -> Result<()> {
        let path = match &self.path {
            Some(p) => p,
            None => bail!("no path"),
        };
        fs::remove_file(&path)?;
        Ok(())
    }

    pub fn write(&self) -> Result<()> {
        let path = match &self.path {
            Some(p) => p,
            None => bail!("no path"),
        };
        log::info!(
            "{} {}/{} ({}%)",
            self.state,
            self.current,
            self.total,
            self.percent()
        );
        let file = fs::File::create(path)?;
        serde_json::to_writer_pretty(file, &self)?;
        Ok(())
    }
    fn percent(&self) -> u64 {
        let percent = self.current as f64 * 100.0 / self.total as f64;
        percent as u64
    }
    pub fn start(&mut self, total: u64) -> Result<()> {
        self.total = total;
        self.current = 0;
        self.write()
    }
    pub fn state(&mut self, state: &str) -> Result<()> {
        self.state = state.to_owned();
        self.write()
    }

    pub fn inc(&mut self) -> Result<()> {
        let p0 = self.percent();
        self.current += 1;
        let p1 = self.percent();
        if p1 > p0 {
            self.write()?;
        }
        Ok(())
    }
    pub fn error(&mut self, error: anyhow::Error) -> Result<()> {
        self.error = Some(format!("Error: {}", error));
        self.write()?;
        Ok(())
    }

    pub fn finish(&mut self) -> Result<()> {
        self.current = self.total;
        self.write()
    }
}
