use anyhow::{bail, Result};
use sqlite::Connection;
use std::collections::HashMap;

use crate::Database;

#[derive(Debug, Clone)]
pub struct SlideData {
    pub tile_size: u64,
    pub levels: u64,
    pub width: u64,
    pub height: u64,
    pub x_ppm: u64,
    pub y_ppm: u64,
}

fn read(db: &Connection, key: &str) -> Result<String> {
    let mut statement = db.prepare("SELECT value FROM metadata WHERE key = ?")?;
    statement.bind((1, key))?;
    let str = match statement.next()? {
        sqlite::State::Row => statement.read::<String, _>(0)?,
        _ => bail!("Failed to read required key {}", key),
    };
    Ok(str)
}

fn write(db: &Connection, key: &str, value: &str) -> Result<()> {
    let mut statement = db.prepare(
        "INSERT INTO metadata (key, value)
                     VALUES (?, ?)
                     ON CONFLICT(key)
                     DO UPDATE SET value=excluded.value;
                ",
    )?;
    statement.bind((1, key))?;
    statement.bind((2, value))?;

    match statement.next()? {
        sqlite::State::Done => return Ok(()),
        _ => bail!("Failed insert"),
    }
}

fn read_u64(db: &Connection, key: &str) -> Result<u64> {
    let str = read(db, key)?;
    let value = str.parse::<u64>()?;
    Ok(value)
}
fn write_u64(db: &Connection, key: &str, value: u64) -> Result<()> {
    let str = value.to_string();
    write(db, key, &str)
}

impl SlideData {
    pub fn new(
        tile_size: u64,
        levels: u64,
        width: u64,
        height: u64,
        x_ppm: u64,
        y_ppm: u64,
    ) -> SlideData {
        SlideData {
            tile_size,
            levels,
            width,
            height,
            x_ppm,
            y_ppm,
        }
    }

    pub fn from(db: &Connection) -> Result<SlideData> {
        Ok(SlideData {
            tile_size: read_u64(db, "tile_size")?,
            levels: read_u64(db, "levels")?,
            width: read_u64(db, "width")?,
            height: read_u64(db, "height")?,
            x_ppm: read_u64(db, "x_ppm")?,
            y_ppm: read_u64(db, "y_ppm")?,
        })
    }
    pub fn write_to(&self, db: &Connection) -> Result<()> {
        write_u64(db, "tile_size", self.tile_size)?;
        write_u64(db, "levels", self.levels)?;
        write_u64(db, "width", self.width)?;
        write_u64(db, "height", self.height)?;
        write_u64(db, "x_ppm", self.x_ppm)?;
        write_u64(db, "y_ppm", self.y_ppm)?;
        Ok(())
    }
}

impl Database {
    pub fn set_meta(&self, key: &str, value: &str) -> Result<()> {
        self.check_writeable()?;
        write(&self.db, key, value)
    }
    pub fn read_metadata(&self) -> Result<HashMap<String, String>> {
        let statement = "SELECT key, value from metadata";
        let mut statement = self.db.prepare(statement)?;
        let mut metadata: HashMap<String, String> = HashMap::new();

        loop {
            match statement.next()? {
                sqlite::State::Row => {
                    let key = statement.read::<String, _>(0)?;
                    let data = statement.read::<String, _>(1)?;
                    metadata.insert(key, data);
                }
                _ => break,
            };
        }
        Ok(metadata)
    }
    pub fn write_metadata(&self, meta: HashMap<String, String>) -> Result<()> {
        for (key, value) in meta.iter() {
            self.set_meta(key, value)?;
        }
        Ok(())
    }
}
