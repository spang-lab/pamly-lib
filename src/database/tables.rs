use super::Database;

use anyhow::{bail, Result};

impl Database {
    pub fn table_exists(&self, name: String) -> Result<bool> {
        let statement = "SELECT name FROM sqlite_master WHERE name= ?";
        let mut statement = self.db.prepare(statement)?;
        statement.bind((1, name.as_str()))?;
        match statement.next()? {
            sqlite::State::Row => return Ok(true),
            _ => return Ok(false),
        }
    }

    pub fn check_tables(&self) -> Result<()> {
        self.check_writeable()?;
        let tables = vec!["tiles", "metadata", "diagnosis_details", "diagnosis"];
        for table_name in tables {
            if !self.table_exists(table_name.to_owned())? {
                match table_name {
                    "tiles" => self.create_tiles_table()?,
                    "metadata" => self.create_metadata_table()?,
                    "labels" => self.create_labels_table()?,
                    _ => bail!("Unknown table name {}", table_name),
                };
            }
        }
        Ok(())
    }

    fn create_tiles_table(&self) -> Result<()> {
        let query = "
            CREATE TABLE tiles (
                x INTEGER,
                y INTEGER,
                level INTEGER,
                type INTEGER,
                jpeg BLOB,
                PRIMARY KEY (x, y, level, type)
            );
        ";
        self.db.execute(query)?;
        Ok(())
    }

    fn create_metadata_table(&self) -> Result<()> {
        let query = "
            CREATE TABLE metadata (key TEXT UNIQUE, value TEXT);
        ";
        self.db.execute(query)?;
        Ok(())
    }

    fn create_labels_table(&self) -> Result<()> {
        let query = "
            CREATE TABLE labels (
                x INTEGER,
                y INTEGER,
                level INTEGER,
                label INTEGER,
                source TEXT,
                unix_time INTEGER
            );
        ";
        self.db.execute(query)?;
        Ok(())
    }
}
