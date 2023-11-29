use anyhow::Result;

use std::fs;
use std::io::Write;
use std::path::PathBuf;

use patho_io::Progress;

pub struct Lock {
    path: PathBuf,
    name: String,
    total: u64,
    current: u64,
    quiet: bool,
}

impl Lock {
    pub fn new(path: PathBuf, quiet: bool) -> Lock {
        Lock {
            path,
            name: "".to_owned(),
            total: 0,
            current: 0,
            quiet,
        }
    }
    pub fn exists(&self) -> bool {
        self.path.exists() && self.path.is_file()
    }

    pub fn create(&self) -> Result<()> {
        fs::File::create(&self.path)?;
        Ok(())
    }
    pub fn name(&mut self, name: String) {
        self.name = name;
    }

    pub fn delete(&self) -> Result<()> {
        fs::remove_file(&self.path)?;
        Ok(())
    }
    fn percent(&self) -> u64 {
        let percent = self.current as f64 * 100.0 / self.total as f64;
        percent as u64
    }

    fn print(&self) -> Result<()> {
        let mut file = fs::File::create(&self.path)?;
        write!(file, "{} {}%", self.name, self.percent())?;
        if !self.quiet {
            println!("{} {}%", self.name, self.percent());
        }
        Ok(())
    }
    pub fn error(&self, error: anyhow::Error) -> Result<()> {
        let mut file = fs::File::create(&self.path)?;
        write!(file, "Error: {}", error)?;
        println!("Error: {}", error);
        Ok(())
    }
}

impl Progress for Lock {
    fn start(&mut self, total: u64) {
        self.total = total;
        self.current = 0;
        self.print().unwrap();
    }
    fn inc(&mut self) {
        let p0 = self.percent();
        self.current += 1;
        let p1 = self.percent();
        if p1 > p0 {
            self.print().unwrap();
        }
    }
    fn finish(&mut self) {
        self.current = self.total;
        self.print().unwrap();
    }
}
