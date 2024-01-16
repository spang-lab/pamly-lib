use anyhow::{bail, Result};
use image::RgbImage;

pub struct Patch {
    data: Option<RgbImage>,
    level: u64,
    pub coords: (u64, u64),
}

impl Patch {
    pub fn new(coords: (u64, u64), level: u64) -> Patch {
        Patch {
            data: None,
            level,
            coords,
        }
    }
    pub fn set_image(&mut self, image: RgbImage) {
        self.data = Some(image);
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_none()
    }

    pub fn image(&self) -> Result<&RgbImage> {
        match &self.data {
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
    pub fn level(&self) -> u64 {
        self.level
    }

    pub fn coords(&self) -> (u64, u64) {
        self.coords
    }
}
