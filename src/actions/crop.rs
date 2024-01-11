use crate::Database;
use anyhow::Result;
use sqlite::Connection;

fn max_tile(db: &Connection, level: u64) -> Result<(u64, u64)> {
    let statement = "SELECT max(x), max(y) from tiles WHERE
            level = ? AND
            type = 0
        ";
    let mut statement = db.prepare(statement)?;
    statement.bind((1, level as i64))?;
    statement.next()?;
    let max_x = statement.read::<i64, _>(0)?;
    let max_y = statement.read::<i64, _>(1)?;
    Ok((max_x as u64, max_y as u64))
}
fn min_tile(db: &Connection, level: u64) -> Result<(u64, u64)> {
    let statement = "SELECT min(x), min(y) from tiles WHERE
            level = ? AND
            type = 0
        ";
    let mut statement = db.prepare(statement)?;
    statement.bind((1, level as i64))?;
    statement.next()?;
    let min_x = statement.read::<i64, _>(0)?;
    let min_y = statement.read::<i64, _>(1)?;
    Ok((min_x as u64, min_y as u64))
}

pub fn crop(db: &Database) -> Result<()> {
    let connection = db.connection();
    let level = db.levels() - 1;
    let tile_size = db.tile_size();

    let min_t = min_tile(connection, level)?;
    let max_t = max_tile(connection, level)?;

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
    let mut statement = connection.prepare(statement)?;
    statement.bind((1, min_t.0 as i64))?;
    statement.bind((2, min_t.1 as i64))?;
    statement.bind((3, new_level as i64))?;
    statement.bind((4, level as i64))?;
    statement.next()?;
    db.set_meta("levels", &(new_level + 1).to_string())?;
    db.set_meta("width", &size.0.to_string())?;
    db.set_meta("height", &size.1.to_string())?;
    Ok(())
}
