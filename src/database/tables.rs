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
        let tables = vec!["tiles", "metadata", "labels", "distances"];
        for table_name in tables {
            if !self.table_exists(table_name.to_owned())? {
                match table_name {
                    "tiles" => self.create_tiles_table()?,
                    "metadata" => self.create_metadata_table()?,
                    "labels" => self.create_labels_table()?,
                    "distances" => self.create_distances_table()?,
                    _ => bail!("Unknown table name {}", table_name),
                };
            }
        }
        Ok(())
    }

    fn create_tiles_table(&self) -> Result<()> {
        let query = "
            CREATE TABLE tiles (
                id INTEGER PRIMARY KEY, 
                x INTEGER,
                y INTEGER,
                level INTEGER,
                jpeg BLOB,
                UNIQUE (x, y, level)
            );
        ";
        self.db.execute(query)?;
        Ok(())
    }

    fn create_distances_table(&self) -> Result<()> {
        let query = "
            CREATE TABLE distances (
                tile1 INTEGER,
                tile2 INTEGER,
                distance REAL,
                UNIQUE(tile1, tile2),
                CHECK (tile1 <> tile2),
                FOREIGN KEY (tile1) REFERENCES tiles(id),
                FOREIGN KEY (tile2) REFERENCES tiles(id) 
            );
        ";
        self.db.execute(query)?;
        let query = "
            CREATE INDEX idx_tile1 ON distances(tile1)
        ";
        self.db.execute(query)?;
        let query = "
            CREATE INDEX idx_tile2 ON distances(tile2)
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
                tile INTEGER,
                label INTEGER,
                source TEXT,
                unix_time INTEGER,
                nn_order INTEGER,
                undo INTEGER DEFAULT 0,
                FOREIGN KEY (tile) REFERENCES tiles(id)
            );
        ";
        self.db.execute(query)?;
        Ok(())
    }
}
