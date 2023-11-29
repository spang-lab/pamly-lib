use crate::{Config, delete_islands, Lock};

use anyhow::Result;
use std::path::PathBuf;
use std::cmp;

use patho_io::{OpenSlide, Patch, Database, Progress};

use image::{RgbImage, imageops, GrayImage, Pixel};
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
    let edge_count: u64 = edges.into_raw().iter().map(|p| {
            if *p == 0 { 0 }
            else { 1 }
    }).sum();
    let edge_percent = (edge_count as f64) / (size * size) as f64;
    return edge_percent > config.min_edge_content;
}

fn write_metadata(db: &Database, levels: u64, size: (u64, u64), resolution: (u64, u64),  c: &Config) -> Result<()> {
    let (width, height) = size;
    let (x_ppm, y_ppm) = resolution;
    db.insert_meta("scan_width".to_owned(),width.to_string())?;
    db.insert_meta("scan_height".to_owned(),height.to_string())?;
    db.insert_meta("resolution_x_ppm".to_owned(),x_ppm.to_string())?;
    db.insert_meta("resolution_y_ppm".to_owned(),y_ppm.to_string())?;
    db.insert_meta("levels".to_owned(),levels.to_string())?;
    db.insert_meta("tile_size".to_owned(),c.tile_size.to_string())?;
    db.insert_meta("edge_detect_size".to_owned(), c.edge_detect_size.to_string())?;
    db.insert_meta("edge_low_threshold".to_owned(), c.edge_low_threshold.to_string())?;
    db.insert_meta("edge_high_threshold".to_owned(), c.edge_high_threshold.to_string())?;
    db.insert_meta("min_edge_content".to_owned(), c.min_edge_content.to_string())?;
    db.insert_meta("dark_threshold".to_owned(), c.dark_threshold.to_string())?;
    db.insert_meta("min_dark_content".to_owned(), c.min_dark_content.to_string())?;

    Ok(())
}

pub fn convert_slide(slide_path: PathBuf, sqlite_path: PathBuf, lock: &mut Lock, config: &Config) -> Result<()> {
    println!("Writing from {:?} to {:?}", slide_path, sqlite_path);
    let slide = OpenSlide::open(slide_path)?;
    let resolution  = slide.get_resolution()?;

    let sqlite = Database::create(&sqlite_path)?;
    let tile_size = config.tile_size;
    let width = slide.width;
    let height = slide.height;

    let tiles_x = (width as f64 / tile_size as f64).ceil() as u64;
    let tiles_y = (height as f64 / tile_size as f64).ceil() as u64;
    let tree_size = cmp::max(tiles_x, tiles_y);
    let levels = 1 + (tree_size as f64).log2().ceil() as u64; 

    write_metadata(&sqlite, levels, (width, height), resolution, config)?;

    let total = tiles_x * tiles_y;
    lock.name("Reading".to_owned());
    lock.start(total);
    for tx in 0..tiles_x{
        for ty in 0..tiles_y {
            lock.inc();
            let x = tx * tile_size;
            let y = ty * tile_size;
            let image = slide.read_region(x as i64, y as i64, tile_size as i64, tile_size as i64)?;
            if !is_valid_image(&image, config){
                continue;
            }
            let patch = Patch::new(image, (tx, ty));
            patch.write_to_db(&sqlite, levels - 1)?;
        }
    }
    lock.finish();

    println!("Finding islands in {:?}...", sqlite_path);
    delete_islands(&sqlite, levels, config)?;

    sqlite.crop()?;
    

    println!("Upscaling {:?}...", sqlite_path);
    lock.name("Upscaling".to_owned());
    sqlite.upscale(lock)?;
    Ok(())
}
