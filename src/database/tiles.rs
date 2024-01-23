use crate::{Database, Tile};
use anyhow::{bail, Ok, Result};
use sqlite::State;

impl Database {
    pub fn read(&self, pos: (u64, u64), level: u64) -> Result<Tile> {
        let tile_size = self.tile_size();
        let mut tile = Tile::new(pos, level, tile_size);
        let (x, y) = pos;
        let statement = "SELECT jpeg from tiles WHERE
            x = ? AND
            y = ? AND
            level = ?
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

    pub fn read_many(&self, start: (u64, u64), end: (u64, u64), level: u64) -> Result<Vec<Tile>> {
        let tile_size = self.tile_size();

        let statement = "SELECT x, y, jpeg from tiles WHERE
            x >= ? AND x < ? AND
            y >= ? AND y < ? AND
            level = ?
        ";
        let mut statement = self.db.prepare(statement)?;
        statement.bind((1, start.0 as i64))?;
        statement.bind((2, end.0 as i64))?;
        statement.bind((3, start.1 as i64))?;
        statement.bind((4, end.1 as i64))?;
        statement.bind((5, level as i64))?;

        let mut tiles = Vec::new();
        while statement.next()? == State::Row {
            let x = statement.read::<i64, _>(0)?;
            let y = statement.read::<i64, _>(1)?;
            let data = statement.read::<Vec<u8>, _>(2)?;
            let mut tile = Tile::new((x as u64, y as u64), level, tile_size);
            tile.set_data(data)?;
            tiles.push(tile);
        }
        Ok(tiles)
    }

    pub fn delete(&self, pos: (u64, u64), level: u64) -> Result<()> {
        let (x, y) = pos;
        let statement = "DELETE from tiles WHERE
            x = ? AND
            y = ? AND
            level = ?
        ";
        let mut statement = self.db.prepare(statement)?;
        statement.bind((1, x as i64))?;
        statement.bind((2, y as i64))?;
        statement.bind((3, level as i64))?;

        statement.next()?;
        Ok(())
    }
    pub fn write(&self, tile: &Tile) -> Result<()> {
        if tile.is_empty() {
            return Ok(());
        }
        let (x, y) = tile.pos();
        let level = tile.level();
        let id = tile.index();
        let data = tile.data()?;
        let statement = "INSERT INTO tiles VALUES (?, ?, ?, ?, ?)";
        let mut statement = self.db.prepare(statement)?;
        statement.bind((1, id as i64))?;
        statement.bind((2, x as i64))?;
        statement.bind((3, y as i64))?;
        statement.bind((4, level as i64))?;
        statement.bind((5, &data[..]))?;

        match statement.next()? {
            sqlite::State::Done => return Ok(()),
            _ => bail!("Failed insert"),
        }
    }

    pub fn list_tiles(&self, level: u64) -> Result<Vec<(u64, u64)>> {
        let statement = "SELECT x, y from tiles WHERE
            level = ?
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
}
