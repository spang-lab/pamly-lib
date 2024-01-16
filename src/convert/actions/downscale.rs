use image::{imageops, ImageBuffer, Rgb, RgbImage};

use anyhow::Result;

use crate::convert::LockFile;
use crate::{Database, Tile};

fn combine_into(tile: &mut Tile, tiles: Vec<Tile>) -> Result<()> {
    let is_empty = tiles.iter().all(|t| t.is_empty());
    if is_empty {
        return Ok(());
    }

    let s = tile.size() as u32;
    let white = Rgb([255, 255, 255]);
    let mut result: RgbImage = ImageBuffer::from_pixel(2 * s, 2 * s, white);

    for dy in 0..2 {
        for dx in 0..2 {
            let tile_idx = dy * 2 + dx;
            let tile = &tiles[tile_idx];
            if tile.is_empty() {
                continue;
            }
            imageops::replace(
                &mut result,
                tile.image()?,
                dx as i64 * s as i64,
                dy as i64 * s as i64,
            );
        }
    }
    let scale_type = imageops::FilterType::Lanczos3;
    let scaled = imageops::resize(&result, s, s, scale_type);
    tile.set_image(scaled)?;
    Ok(())
}

fn compute_tile(db: &Database, tile: &mut Tile, lock: &mut LockFile) -> Result<()> {
    lock.inc()?;

    let pos = tile.pos();
    let level = tile.level();
    let size = tile.size();
    let levels = db.levels();

    let existing_tile = db.read(pos, level)?;
    if !existing_tile.is_empty() {
        let image = existing_tile.image()?;
        tile.set_image(image.clone())?;
    }
    if !existing_tile.is_empty() || level == levels - 1 {
        return Ok(());
    }

    let mut sub_tiles = Vec::new();
    let sub_level = level + 1;
    for dy in 0..2 {
        for dx in 0..2 {
            let sx = 2 * pos.0 + dx;
            let sy = 2 * pos.1 + dy;
            let mut sub_tile = Tile::new((sx, sy), sub_level, size);
            compute_tile(db, &mut sub_tile, lock)?;
            sub_tiles.push(sub_tile);
        }
    }
    combine_into(tile, sub_tiles)?;
    db.write(tile)?;
    Ok(())
}

pub fn downscale(db: &Database, lock: &mut LockFile) -> Result<()> {
    let mut total_nodes: u64 = 0;
    let base: u64 = 4;
    let levels = db.levels();
    let tile_size = db.tile_size();

    for i in 0..levels {
        total_nodes += base.pow(i as u32);
    }
    lock.state("Downscaling")?;
    lock.start(total_nodes)?;
    let mut root_tile = Tile::new((0, 0), 0, tile_size);
    compute_tile(db, &mut root_tile, lock)?;
    lock.finish()?;
    Ok(())
}
