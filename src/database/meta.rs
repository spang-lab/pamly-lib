use anyhow::{bail, Result};
use sqlite::Connection;

use crate::Database;

#[derive(Debug)]
pub struct Metadata {
    pub tile_size: u64,
    pub levels: u64,
    pub width: u64,
    pub height: u64,
    pub x_ppm: u64,
    pub y_ppm: u64,
}

fn read_u64(db: &Connection, key: &str) -> Result<u64> {
    let mut statement = db.prepare("SELECT value FROM metadata WHERE key = ?")?;
    statement.bind((1, key))?;
    let str = match statement.next()? {
        sqlite::State::Row => statement.read::<String, _>(0)?,
        _ => bail!("Failed to read required key {}", key),
    };
    let value = str.parse::<u64>()?;
    Ok(value)
}

impl Metadata {
    pub fn from(db: &Connection) -> Result<Metadata> {
        Ok(Metadata {
            tile_size: read_u64(db, "width")?,
            levels: read_u64(db, "levels")?,
            width: read_u64(db, "width")?,
            height: read_u64(db, "height")?,
            x_ppm: read_u64(db, "resolution_x_ppm")?,
            y_ppm: read_u64(db, "resolution_y_ppm")?,
        })
    }
}

impl Database {
    pub fn set_meta(&self, key: String, value: String) -> Result<()> {
        self.check_writeable()?;
        let mut statement = self.db.prepare(
            "INSERT INTO metadata (key, value)
                     VALUES (?, ?)
                     ON CONFLICT(key)
                     DO UPDATE SET value=excluded.value;
                ",
        )?;
        statement.bind((1, key.as_str()))?;
        statement.bind((2, value.as_str()))?;

        match statement.next()? {
            sqlite::State::Done => return Ok(()),
            _ => bail!("Failed insert"),
        }
    }
}
