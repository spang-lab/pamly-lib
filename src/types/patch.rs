use anyhow::{bail, Result};
use image::RgbImage;

pub struct Patch {
    data: Option<RgbImage>,
    pos: (u64, u64),
}

impl Patch {
    pub fn new(pos: (u64, u64)) -> Tile {
        Tile {
            data: None,
            level,
            pos,
        }
    }
    pub fn set_data(&mut self, data: Vec<u8>) -> Result<()> {
        let dyn_img = image::load_from_memory(&data)?;
        let rgb_img = dyn_img.to_rgb8();
        self.data = Some(rgb_img);
        Ok(())
    }
    pub fn is_empty(&self) -> bool {
        self.data.is_none()
    }

    pub fn image(&self) -> Result<RgbImage> {
        match self.data {
            Some(d) => Ok(d),
            None => bail!("Patch is empty"),
        }
    }

    pub fn size(&self) -> Result<(u64, u64)> {
        let img = self.image()?;
        let h = img.height() as u64;
        let w = img.width() as u64;
        Ok((w, h))
    }

    pub fn coords(&self) -> (u64, u64) {
        self.pos
    }
}
