use crate::Database;
use anyhow::Result;

fn edges(db: &Database) -> Result<(u64, u64, u64, u64)> {
    let level = db.levels() - 1;
    let statement = "SELECT min(x), max(x), min(y), max(y) from tiles WHERE
            level = ?
        ";
    let mut statement = db.connection().prepare(statement)?;
    statement.bind((1, level as i64))?;
    statement.next()?;
    let min_x = statement.read::<i64, _>(0)? as u64;
    let max_x = statement.read::<i64, _>(1)? as u64;
    let min_y = statement.read::<i64, _>(2)? as u64;
    let max_y = statement.read::<i64, _>(3)? as u64;
    Ok((min_x, max_x, min_y, max_y))
}

fn move_tiles(db: &Database, dx: u64, dy: u64, new_level: u64) -> Result<()> {
    let level = db.levels() - 1;
    let statement = "UPDATE tiles
            SET x = x - ?,
                y = y - ?,
                level = ?
            WHERE
                level = ?
        ";
    let mut statement = db.connection().prepare(statement)?;
    statement.bind((1, dx as i64))?;
    statement.bind((2, dy as i64))?;
    statement.bind((3, new_level as i64))?;
    statement.bind((4, level as i64))?;
    statement.next()?;
    Ok(())
}

pub fn crop(db: &mut Database) -> Result<()> {
    let tile_size = db.tile_size();

    let (min_x, max_x, min_y, max_y) = edges(db)?;

    let tile_count = (max_x - min_x + 1, max_y - min_y + 1);
    let size = (tile_count.0 * tile_size, tile_count.1 * tile_size);

    let tree_size = std::cmp::max(tile_count.0, tile_count.1);
    let new_level = (tree_size as f64).log2().ceil() as u64;
    move_tiles(db, min_x, min_y, new_level)?;

    log::debug!(
        "Cropping Slide from {}x{} to {}x{}",
        db.width(),
        db.height(),
        size.0,
        size.1
    );

    db.data.levels = new_level + 1;
    db.data.width = size.0;
    db.data.height = size.1;

    db.set_meta("levels", &(new_level + 1).to_string())?;
    db.set_meta("width", &size.0.to_string())?;
    db.set_meta("height", &size.1.to_string())?;
    Ok(())
}
