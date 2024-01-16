use crate::convert::{Config, LockFile, OpenSlide};
use crate::{Database, Tile};
use anyhow::Result;

use image::{imageops, GrayImage, Pixel, RgbImage};
use imageops::FilterType;
use imageproc::edges::canny;

pub fn is_valid_image(image: &RgbImage, config: &Config) -> bool {
    let (w, h) = image.dimensions();
    let mut gray = GrayImage::new(w, h);

    let threshold = (255.0 * config.dark_threshold) as u8;
    let mut dark_pixels = 0;

    for (g_p, rgb_p) in gray.pixels_mut().zip(image.pixels()) {
        *g_p = rgb_p.to_luma();
        if g_p[0] < threshold {
            dark_pixels += 1;
        }
    }
    let dark_content = dark_pixels as f64 / (w as f64 * h as f64);
    if dark_content < config.min_dark_content {
        return false;
    }

    let size = config.edge_detect_size as u32;
    let resized = imageops::resize(&gray, size, size, FilterType::Triangle);
    let edges = canny(
        &resized,
        config.edge_low_threshold,
        config.edge_high_threshold,
    );
    let edge_count: u64 = edges
        .into_raw()
        .iter()
        .map(|p| if *p == 0 { 0 } else { 1 })
        .sum();
    let edge_percent = (edge_count as f64) / (size * size) as f64;
    return edge_percent > config.min_edge_content;
}

pub fn read_slide(
    slide: &OpenSlide,
    db: &Database,
    config: &Config,
    lock: &mut LockFile,
) -> Result<()> {
    let tile_size = db.tile_size();
    let width = db.width();
    let height = db.height();
    let levels = db.levels();
    let tiles_x = (width as f64 / tile_size as f64).ceil() as u64;
    let tiles_y = (height as f64 / tile_size as f64).ceil() as u64;

    let total = tiles_x * tiles_y;
    lock.state("Reading")?;
    lock.start(total)?;

    for tx in 0..tiles_x {
        for ty in 0..tiles_y {
            lock.inc()?;
            let mut tile = Tile::new((tx, ty), levels - 1, tile_size);
            let (x, y) = tile.coords();
            let image =
                slide.read_region(x as i64, y as i64, tile_size as i64, tile_size as i64)?;
            if !is_valid_image(&image, config) {
                continue;
            }
            tile.set_image(image)?;
            db.write(&tile)?;
        }
    }
    lock.finish()?;
    Ok(())
}
