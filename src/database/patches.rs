use crate::{Database, Patch};
use anyhow::{bail, Result};
use image::{imageops, ImageBuffer, Rgb};

impl Database {
    pub fn read_region(&self, pos: (u64, u64), size: (u64, u64)) -> Result<Patch> {
        self.read_region_level(pos, size, self.levels() - 1)
    }

    fn read_region_level(&self, coords: (u64, u64), size: (u64, u64), level: u64) -> Result<Patch> {
        let tile_size = self.tile_size();
        let start = (
            coords.0.div_euclid(tile_size),
            coords.1.div_euclid(tile_size),
        );
        let end = (
            (coords.0 + size.0).div_ceil(tile_size),
            (coords.1 + size.1).div_ceil(tile_size),
        );
        let tile_count = (end.0 - start.0, end.1 - start.1);
        let read_size = (tile_count.0 * tile_size, tile_count.1 * tile_size);

        let white = Rgb([255, 255, 255]);
        let mut read_buffer =
            ImageBuffer::from_pixel(read_size.0 as u32, read_size.1 as u32, white);

        let mut patch = Patch::new(coords, level);

        let tiles = self.read_many(start, end, level)?;
        if tiles.len() == 0 {
            return Ok(patch);
        }
        for tile in tiles {
            let tile_image = tile.image()?;
            let (x, y) = tile.pos();
            let (dx, dy) = ((x - start.0) * tile_size, (y - start.1) * tile_size);
            imageops::replace(&mut read_buffer, tile_image, dx as i64, dy as i64);
        }

        let crop_x = coords.0 % tile_size;
        let crop_y = coords.1 % tile_size;
        let cropped = imageops::crop(
            &mut read_buffer,
            crop_x as u32,
            crop_y as u32,
            size.0 as u32,
            size.1 as u32,
        )
        .to_image();
        patch.set_image(cropped);
        Ok(patch)
    }

    pub fn read_region_scaled(
        &self,
        coords: (u64, u64),
        size: (u64, u64),
        target_size: (u64, u64),
    ) -> Result<Patch> {
        let max_level = self.levels() - 1;
        let scaling_factor_x = target_size.0 as f64 / size.0 as f64;
        let scaling_factor_y = target_size.1 as f64 / size.1 as f64;
        if (scaling_factor_x - scaling_factor_y).abs() > 1e-3 {
            bail!("Scaling from {:?} to {:?} not supported", size, target_size);
        }
        let level_change = scaling_factor_x.log2().ceil() as i64;
        let level = (max_level as i64 + level_change).clamp(0, max_level as i64) as u64;
        let scale = 2u64.pow((max_level - level) as u32);

        let rel_coords = (coords.0.div_euclid(scale), coords.1.div_euclid(scale));
        let rel_size = (size.0.div_euclid(scale), size.1.div_euclid(scale));

        let patch = self.read_region_level(rel_coords, rel_size, level)?;

        let mut result = Patch::new(coords, level);

        if patch.is_empty() {
            return Ok(result);
        }

        let image = patch.image()?;
        let scaled = imageops::resize(
            image,
            target_size.0 as u32,
            target_size.1 as u32,
            imageops::Lanczos3,
        );
        result.set_image(scaled);
        Ok(result)
    }

    pub fn thumbnail(&self, target_size: u64) -> Result<Patch> {
        let pos = (0, 0);
        let w = self.width();
        let h = self.height();

        let longer_side = std::cmp::max(w, h);
        let scaling = target_size as f64 / longer_side as f64;

        let tx = (w as f64 * scaling).ceil() as u64;
        let ty = (h as f64 * scaling).ceil() as u64;

        self.read_region_scaled(pos, (w, h), (tx, ty))
    }
}
