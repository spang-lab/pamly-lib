use super::Database;
use crate::Tile;
use anyhow::{bail, Result};
use sqlite::State;

impl Database {
    pub fn read(&self, pos: (u64, u64), level: u64) -> Result<Tile> {
        let mut tile = Tile::new(pos, level);

        let (x, y) = pos;
        let statement = "SELECT jpeg from tiles WHERE
            x = ? AND
            y = ? AND
            level = ? AND
            type = 0
        ";
        let mut statement = self.db.prepare(statement)?;
        statement.bind((1, x as i64))?;
        statement.bind((2, y as i64))?;
        statement.bind((3, level as i64))?;

        match statement.next()? {
            State::Row => {
                let data = statement.read::<Vec<u8>, _>(0)?;
                tile.set_data(data)?;
            }
            _ => {}
        };
        return Ok(tile);
    }
    pub fn delete(&self, pos: (u64, u64), level: u64) -> Result<()> {
        let (x, y) = pos;
        let statement = "DELETE from tiles WHERE
            x = ? AND
            y = ? AND
            level = ? AND
            type = 0
        ";
        let mut statement = self.db.prepare(statement)?;
        statement.bind((1, x as i64))?;
        statement.bind((2, y as i64))?;
        statement.bind((3, level as i64))?;

        statement.next()?;
        Ok(())
    }
    pub fn write(&self, pos: (u64, u64), level: u64, jpeg: Vec<u8>) -> Result<()> {
        let (x, y) = pos;
        let base_type = 0;
        let statement = "INSERT INTO tiles VALUES (?, ?, ?, ?, ?)";
        let mut statement = self.db.prepare(statement)?;
        statement.bind((1, x as i64))?;
        statement.bind((2, y as i64))?;
        statement.bind((3, level as i64))?;
        statement.bind((4, base_type))?;
        statement.bind((5, &jpeg[..]))?;

        match statement.next()? {
            sqlite::State::Done => return Ok(()),
            _ => bail!("Failed insert"),
        }
    }

    pub fn list_tiles(&self, level: u64) -> Result<Vec<(u64, u64)>> {
        let statement = "SELECT x, y from tiles WHERE
            level = ? AND
            type = 0
        ";
        let mut statement = self.db.prepare(statement)?;
        statement.bind((1, level as i64))?;

        let mut positions = Vec::new();
        while statement.next()? == State::Row {
            let x = statement.read::<i64, _>(0)?;
            let y = statement.read::<i64, _>(1)?;
            positions.push((x as u64, y as u64));
        }
        Ok(positions)
    }

    fn max_tile(&self, level: u64) -> Result<(u64, u64)> {
        let statement = "SELECT max(x), max(y) from tiles WHERE
            level = ? AND
            type = 0
        ";
        let mut statement = self.db.prepare(statement)?;
        statement.bind((1, level as i64))?;
        statement.next()?;
        let max_x = statement.read::<i64, _>(0)?;
        let max_y = statement.read::<i64, _>(1)?;
        Ok((max_x as u64, max_y as u64))
    }
    fn min_tile(&self, level: u64) -> Result<(u64, u64)> {
        let statement = "SELECT min(x), min(y) from tiles WHERE
            level = ? AND
            type = 0
        ";
        let mut statement = self.db.prepare(statement)?;
        statement.bind((1, level as i64))?;
        statement.next()?;
        let min_x = statement.read::<i64, _>(0)?;
        let min_y = statement.read::<i64, _>(1)?;
        Ok((min_x as u64, min_y as u64))
    }

    pub fn crop(&self) -> Result<()> {
        let level = self.levels()? - 1;
        let tile_size = self.tile_size()?;
        let min_t = self.min_tile(level)?;
        let max_t = self.max_tile(level)?;

        let tile_count = (max_t.0 - min_t.0 + 1, max_t.1 - min_t.1 + 1);
        let size = (tile_count.0 * tile_size, tile_count.1 * tile_size);

        let tree_size = std::cmp::max(tile_count.0, tile_count.1);
        let new_level = (tree_size as f64).log2().ceil() as u64;

        let statement = "UPDATE tiles
            SET x = x - ?,
                y = y - ?,
                level = ?
            WHERE
                level = ? AND
                type = 0
        ";
        let mut statement = self.db.prepare(statement)?;
        statement.bind((1, min_t.0 as i64))?;
        statement.bind((2, min_t.1 as i64))?;
        statement.bind((3, new_level as i64))?;
        statement.bind((4, level as i64))?;
        statement.next()?;
        self.set_meta("levels".to_owned(), (new_level + 1).to_string())?;
        self.insert_meta("width".to_owned(), size.0.to_string())?;
        self.insert_meta("height".to_owned(), size.1.to_string())?;
        Ok(())
    }
}
