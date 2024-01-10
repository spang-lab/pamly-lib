use anyhow::{bail, Result};
use sqlite::{Connection, OpenFlags};
use std::path::PathBuf;

use super::meta::Metadata;

pub struct Database {
    pub db: Connection,
    path: PathBuf,
    meta: Option<Metadata>,
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
            meta: None,
        };
        database.check_tables()?;
        Ok(database)
    }

    pub fn new(path: &PathBuf) -> Result<Database> {
        let flags = OpenFlags::new().with_read_write();
        let db = Connection::open_with_flags(&path, flags)?;
        let meta = Metadata::from(&db)?;
        Ok(Database {
            db,
            path: path.clone(),
            writeable: true,
            meta: Some(meta),
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
        let meta = Metadata::from(&db)?;
        Ok(Database {
            db,
            path: path.clone(),
            writeable: false,
            meta: Some(meta),
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
    pub fn meta(&self) -> Result<&Metadata> {
        match &self.meta {
            Some(m) => Ok(m),
            None => bail!("Cannot read empty metadata"),
        }
    }

    pub fn connection(&self) -> &Connection {
        &self.db
    }
}
