use image::{RgbImage, Rgb, ImageBuffer, imageops};


use anyhow::Result;

use crate::{Database, Patch, Progress};


fn combine(patches: Vec<Patch>, pos: (u64, u64)) -> Patch {
    let non_empty = patches.iter().find(|p| !p.is_empty());
    if non_empty.is_none() {
        return Patch::empty(pos);
    }
    let (w, h) = non_empty.unwrap().size();

    let mut buffers = Vec::new();
    let white = Rgb([255, 255, 255]);
    for patch in patches.into_iter() {
        match patch.into_raw() {
            Some(i) => buffers.push(i),
            None => buffers.push(ImageBuffer::from_pixel(w, h, white)),
        }
    }

    let mut result: RgbImage = ImageBuffer::new(2 * w, 2 * h);
    for dy in 0..2 {
        for dx in 0..2 {
            let i_img = dy * 2 + dx;
            imageops::replace(
                &mut result,
                &buffers[i_img],    
                dx as i64 * w as i64,
                dy as i64 * h as i64,
            );
        }
    }
    let scale_type = imageops::FilterType::Lanczos3;
    let scaled = imageops::resize(&result, w, h, scale_type);
    Patch::new(scaled, pos)
}



impl Database {
    pub fn upscale (&self, lock: &mut dyn Progress) -> Result<()> {
        let mut total_nodes:u64 = 0;
        let base:u64 = 4;
        let levels = self.levels()?;
        for i in 0..levels {
            total_nodes += base.pow(i as u32);
        }
        lock.start(total_nodes);
        self.recursive_combine((0, 0), 0, levels, lock)?;
        lock.finish();
        Ok(())
    }

    fn recursive_combine(&self , pos: (u64, u64), level: u64, levels: u64, lock: &mut dyn Progress) -> Result<Patch> {
        lock.inc();
        let patch = Patch::read_from_db(&self, pos, level)?;
        if !patch.is_empty() {
            return Ok(patch); 
        }
        if level == levels - 1 {
           return Ok(Patch::empty(pos)); 
        }
        let mut sub_images = Vec::new();
        let sub_level = level + 1;
        for dy in 0..2 {
            for dx in 0..2 {
                let sx = 2 * pos.0 + dx;
                let sy = 2 * pos.1 + dy;
                sub_images.push(self.recursive_combine((sx, sy), sub_level, levels, lock)?);
            }
        }
        let cpatch = combine(sub_images, pos);
        cpatch.clone().write_to_db(&self, level)?;
        Ok(cpatch)
    }
}




