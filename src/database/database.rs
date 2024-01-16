use anyhow::{bail, Result};
use sqlite::{Connection, OpenFlags};
use std::path::PathBuf;

use super::SlideData;

pub struct Database {
    pub db: Connection,
    path: PathBuf,
    pub data: SlideData,
    writeable: bool,
}

impl Database {
    pub fn create(path: &PathBuf, data: SlideData) -> Result<Database> {
        let flags = OpenFlags::new().with_read_write().with_create();
        let db = Connection::open_with_flags(&path, flags)?;
        let database = Database {
            db,
            path: path.clone(),
            writeable: true,
            data: data.clone(),
        };
        database.check_tables()?;
        data.write_to(&database.db)?;
        Ok(database)
    }

    pub fn open(path: &PathBuf) -> Result<Database> {
        Database::open_readonly(path)
    }

    pub fn open_readwrite(path: &PathBuf) -> Result<Database> {
        let flags = OpenFlags::new().with_read_write();
        let db = Connection::open_with_flags(&path, flags)?;
        let data = SlideData::from(&db)?;
        Ok(Database {
            db,
            path: path.clone(),
            writeable: true,
            data,
        })
    }
    pub fn open_readonly(path: &PathBuf) -> Result<Database> {
        let flags = OpenFlags::new().with_read_only();
        let db = Connection::open_with_flags(path, flags)?;
        let data = SlideData::from(&db)?;
        Ok(Database {
            db,
            path: path.clone(),
            writeable: false,
            data,
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
    pub fn tile_size(&self) -> u64 {
        self.data.tile_size
    }
    pub fn levels(&self) -> u64 {
        self.data.levels
    }
    pub fn width(&self) -> u64 {
        self.data.width
    }
    pub fn height(&self) -> u64 {
        self.data.height
    }

    pub fn connection(&self) -> &Connection {
        &self.db
    }
}
