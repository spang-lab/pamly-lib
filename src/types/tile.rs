use anyhow::{bail, Result};
use image::{DynamicImage, ImageOutputFormat, RgbImage};
use std::io::Cursor;

pub struct Tile {
    data: Option<RgbImage>,
    size: u64,
    level: u64,
    pub pos: (u64, u64),
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
    pub fn data(&self) -> Result<Vec<u8>> {
        let image = self.image()?;
        let mut bytes = Cursor::new(Vec::new());
        let format = ImageOutputFormat::Jpeg(90);
        let img = DynamicImage::ImageRgb8(image.clone());
        img.write_to(&mut bytes, format)?;
        Ok(bytes.into_inner())
    }

    pub fn size(&self) -> u64 {
        self.size
    }
    pub fn level(&self) -> u64 {
        self.level
    }
    pub fn pos(&self) -> (u64, u64) {
        self.pos
    }
    pub fn index(&self) -> u64 {
        let l = self.level as u32;
        let (x, y) = self.pos;
        (4u64.pow(l) - 1).div_euclid(3) + y * 2u64.pow(self.level as u32) + x
    }

    pub fn coords(&self) -> (u64, u64) {
        let (x, y) = self.pos;
        (x * self.size, y * self.size)
    }
}
