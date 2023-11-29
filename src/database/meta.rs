use super::Database;
use std::collections::HashMap;

use anyhow::{bail, Result};

use sqlite::State;

impl Database {
    pub fn read_metadata(&self) -> Result<HashMap<String, String>> {
        let statement = "SELECT key, value from metadata";
        let mut statement = self.db.prepare(statement)?;
        let mut metadata: HashMap<String, String> = HashMap::new();

        loop {
            match statement.next()? {
                State::Row => {
                    let key = statement.read::<String, _>(0)?;
                    let data = statement.read::<String, _>(1)?;
                    metadata.insert(key, data);
                }
                _ => break,
            };
        }
        Ok(metadata)
    }

    pub fn read_meta(&self, key: String) -> Result<String> {
        let mut statement = self
            .db
            .prepare("SELECT value FROM metadata WHERE key = ?")?;
        statement.bind((1, key.as_str()))?;
        match statement.next()? {
            sqlite::State::Row => return Ok(statement.read::<String, _>(0)?),
            _ => bail!("Failed to read key {}", key),
        }
    }
    pub fn set_meta(&self, key: String, value: String) -> Result<()> {
        self.check_writeable()?;
        let mut statement = self.db.prepare(
            "UPDATE metadata 
                SET value = ?
                WHERE key = ? ",
        )?;
        statement.bind((1, key.as_str()))?;
        statement.bind((2, value.as_str()))?;
        match statement.next()? {
            sqlite::State::Done => return Ok(()),
            _ => bail!("Failed set"),
        }
    }

    pub fn datasets(&self) -> Result<Vec<String>> {
        let mut datasets = Vec::new();
        let metadata = self.read_metadata()?;
        for (key, _) in metadata {
            match key.split_once("dataset:") {
                Some((_, dataset)) => datasets.push(dataset.to_owned()),
                None => {}
            }
        }
        Ok(datasets)
    }

    pub fn insert_meta(&self, key: String, value: String) -> Result<()> {
        self.check_writeable()?;
        let mut statement = self
            .db
            .prepare("INSERT INTO metadata (key, value) VALUES (?, ?);")?;
        statement.bind((1, key.as_str()))?;
        statement.bind((2, value.as_str()))?;

        match statement.next()? {
            sqlite::State::Done => return Ok(()),
            _ => bail!("Failed insert"),
        }
    }

    pub fn check_integrity(&self) -> Result<()> {
        self.check_writeable()?;
        let mut statement = self.db.prepare("pragma integrity_check")?;
        match statement.next()? {
            State::Row => {
                let result = statement.read::<String, _>(0)?;
                if result != "ok" {
                    bail!("Database is not ok. {}", result);
                }
            }
            State::Done => bail!("Got no check result"),
        }
        self.insert_meta("validated".to_owned(), "true".to_owned())?;
        Ok(())
    }

    pub fn levels(&self) -> Result<u64> {
        let levels_str = self.read_meta("levels".to_owned())?;
        let levels = levels_str.parse::<u64>()?;
        Ok(levels)
    }
    pub fn tile_size(&self) -> Result<u64> {
        let ts_str = self.read_meta("tile_size".to_owned())?;
        let ts = ts_str.parse::<u64>()?;
        Ok(ts)
    }
}
