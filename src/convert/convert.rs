use std::path::PathBuf;

use anyhow::Result;

use super::actions;
use super::LockFile;

use crate::database::SlideData;
use crate::Database;

use super::{Config, OpenSlide};

pub fn convert(slide_path: PathBuf, db_path: PathBuf, config: &Config) -> Result<()> {
    let slide = OpenSlide::open(&slide_path)?;

    let tile_size = config.tile_size;

    let (x_ppm, y_ppm) = slide.get_resolution()?;
    let (width, height) = slide.size();

    let tiles_x = (width as f64 / tile_size as f64).ceil() as u64;
    let tiles_y = (height as f64 / tile_size as f64).ceil() as u64;
    let tree_size = std::cmp::max(tiles_x, tiles_y);
    let levels = 1 + (tree_size as f64).log2().ceil() as u64;

    log::debug!("Detected slide from vendor '{}'", slide.vendor);
    log::debug!("  size (w x h): {}x{}", slide.width, slide.height);
    log::debug!("  resolution:   {}x{}", x_ppm, y_ppm);
    log::debug!("  tiles:        {}x{}", tiles_x, tiles_y);
    log::debug!("  levels:       {}", levels);

    let slide_data = SlideData::new(tile_size, levels, width, height, x_ppm, y_ppm);
    let mut db = Database::create(&db_path, slide_data)?;

    let mut config_map = config.to_hash_map()?;
    let path_str = std::fs::canonicalize(slide_path)?
        .to_string_lossy()
        .to_string();
    config_map.insert("slide_path".to_owned(), path_str);

    let mut lock = LockFile::lock(&db_path, "Init")?;
    actions::read_slide(&slide, &db, config, &mut lock)?;
    actions::remove_islands(&db, config, &mut lock)?;
    actions::crop(&mut db)?;
    actions::downscale(&db, &mut lock)?;

    db.write_metadata(config_map)?;
    lock.release()?;

    Ok(())
}
