use anyhow::{bail, Result};
use sqlite::{Connection, OpenFlags};
use std::path::PathBuf;

pub struct Database {
    pub db: Connection,
    path: PathBuf,
    writeable: bool,
}

impl Database {
    pub fn create(path: &PathBuf) -> Result<Database> {
        let flags = OpenFlags::new().with_read_write().with_create();
        let db = Connection::open_with_flags(&path, flags)?;
        let database = Database {
            db,
            path: path.clone(),
            writeable: true,
        };
        database.check_tables()?;
        Ok(database)
    }

    pub fn new(path: &PathBuf) -> Result<Database> {
        let flags = OpenFlags::new().with_read_write();
        let db = Connection::open_with_flags(&path, flags)?;
        Ok(Database {
            db,
            path: path.clone(),
            writeable: true,
        })
    }
    pub fn open(path: &PathBuf) -> Result<Database> {
        Database::open_readonly(path)
    }

    pub fn open_readwrite(path: &PathBuf) -> Result<Database> {
        Self::new(path)
    }
    pub fn open_readonly(path: &PathBuf) -> Result<Database> {
        let flags = OpenFlags::new().with_read_only();
        let db = Connection::open_with_flags(path, flags)?;
        Ok(Database {
            db,
            path: path.clone(),
            writeable: false,
        })
    }
    pub fn is_writeable(&self) -> bool {
        self.writeable
    }
    pub fn check_writeable(&self) -> Result<()> {
        if !self.is_writeable() {
            bail!("Database {} is not writeable", self.path.display())
        }
        Ok(())
    }
}
