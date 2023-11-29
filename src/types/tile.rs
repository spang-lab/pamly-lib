use anyhow::{bail, Result};
use image::RgbImage;

pub struct Tile {
    data: Option<RgbImage>,
    size: u64,
    level: u64,
    pos: (u64, u64),
}

impl Tile {
    pub fn new(pos: (u64, u64), level: u64, size: u64) -> Tile {
        Tile {
            data: None,
            level,
            size,
            pos,
        }
    }

    pub fn set_image(&mut self, img: RgbImage) -> Result<()> {
        let w = img.width();
        let h = img.height();
        let s = self.size as u32;
        if w != s || h != s {
            bail!("invalid image size, is {}x{} should be {}x{}", w, h, s, s);
        }
        self.data = Some(img);
        Ok(())
    }
    pub fn set_data(&mut self, data: Vec<u8>) -> Result<()> {
        let dyn_img = image::load_from_memory(&data)?;
        let rgb_img = dyn_img.to_rgb8();
        self.set_image(rgb_img)
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_none()
    }

    pub fn image(&self) -> Result<&RgbImage> {
        match &self.data {
            Some(d) => Ok(d),
            None => bail!("Tile is empty"),
        }
    }

    pub fn size(&self) -> u64 {
        self.size
    }

    pub fn coords(&self) -> (u64, u64) {
        let (x, y) = self.pos;
        (x * self.size, y * self.size)
    }
}
