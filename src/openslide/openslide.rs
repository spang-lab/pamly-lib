use anyhow::{bail, Result};

use image::{ImageBuffer, RgbImage};

use super::bindings;
use std::path::PathBuf;

const LEVEL: i32 = 0;
// const X_RESOLUTION_KEY:&'static str = "tiff.XResolution";
// const Y_RESOLUTION_KEY:&'static str = "tiff.YResolution";

const X_RESOLUTION_KEY: &'static str = "openslide.mpp-x";
const Y_RESOLUTION_KEY: &'static str = "openslide.mpp-y";

#[derive(Debug, Clone)]
pub struct OpenSlide {
    osr: *const bindings::OpenSlideT,
    pub width: u64,
    pub height: u64,
    pub downsample: f64,
    pub vendor: String,
}
unsafe impl Send for OpenSlide {}
unsafe impl Sync for OpenSlide {}

impl OpenSlide {
    pub fn open(path: PathBuf) -> Result<OpenSlide> {
        let raw_path = match path.to_str() {
            Some(p) => p,
            None => bail!("Path utf8 coding error"),
        };
        let vendor = match bindings::detect_vendor(raw_path) {
            Ok(vendor) => vendor,
            Err(_) => "Unknown".to_owned(),
        };
        let osr = match bindings::open(raw_path) {
            Ok(osr) => osr,
            Err(_) => bail!("Failed to open slide"),
        };
        let downsample = match unsafe { bindings::get_level_downsample(osr, LEVEL) } {
            Ok(f) => f,
            Err(_) => bail!("Failed to get downsample factor"),
        };
        let (width, height) = match unsafe { bindings::get_level_dimensions(osr, LEVEL) } {
            Ok((w, h)) => (w, h),
            Err(_) => bail!("Failed to read slide dimensions"),
        };
        Ok(OpenSlide {
            osr,
            vendor,
            downsample,
            width: width as u64,
            height: height as u64,
        })
    }
    pub fn read_region(&self, x: i64, y: i64, width: i64, height: i64) -> Result<RgbImage> {
        let data = match unsafe { bindings::read_region(self.osr, x, y, LEVEL, width, height) } {
            Ok(r) => r,
            Err(_) => bail!("Call to read_region failed"),
        };
        let mut rgb: Vec<u8> = Vec::with_capacity(data.len() * 3);
        for value in data.iter() {
            let [a, r, g, b] = value.to_be_bytes();
            match a {
                0 => rgb.extend_from_slice(&[0xFF, 0xFF, 0xFF]),
                0xFF => rgb.extend_from_slice(&[r, g, b]),
                _ => rgb.extend_from_slice(&[0xFF * r / a, 0xFF * g / a, 0xFF * b / a]),
            };
        }
        let image_buffer: RgbImage = match ImageBuffer::from_raw(width as u32, height as u32, rgb) {
            Some(image) => image,
            None => bail!("Could not convert tile to image buffer"),
        };

        Ok(image_buffer)
    }

    pub fn print_metadata(&self) -> Result<()> {
        let names = match unsafe { bindings::get_property_names(self.osr) } {
            Ok(n) => n,
            Err(_) => bail!("Failed to read property names."),
        };
        for name in names {
            let value = match unsafe { bindings::get_property_value(self.osr, &name) } {
                Ok(v) => v,
                Err(_) => bail!("Failed to get_property_value {}", name),
            };
            println!("{} = {}", name, value)
        }
        return Ok(());
    }

    // returns slide resolution in pixels per meter
    pub fn get_resolution(&self) -> Result<(u64, u64)> {
        let names = match unsafe { bindings::get_property_names(self.osr) } {
            Ok(n) => n,
            Err(_) => bail!("Failed to read property names."),
        };
        let x_key = X_RESOLUTION_KEY.to_owned();
        let y_key = Y_RESOLUTION_KEY.to_owned();
        if !names.contains(&x_key) || !names.contains(&y_key) {
            bail!("Could not read resolution from slide");
        }
        let x_res_str = match unsafe { bindings::get_property_value(self.osr, &x_key) } {
            Ok(x) => x,
            Err(_) => bail!("Failed to read property value {}", &x_key),
        };
        let y_res_str = match unsafe { bindings::get_property_value(self.osr, &y_key) } {
            Ok(y) => y,
            Err(_) => bail!("Failed to read property value {}", &y_key),
        };
        let x_mpp = x_res_str.parse::<f64>()?;
        let y_mpp = y_res_str.parse::<f64>()?;

        let x_res = (1_000_000.0 / x_mpp) as u64;
        let y_res = (1_000_000.0 / y_mpp) as u64;
        Ok((x_res, y_res))
    }

    pub fn size(&self) -> (u64, u64) {
        (self.width, self.height)
    }
}
impl Drop for OpenSlide {
    fn drop(&mut self) {
        unsafe { bindings::close(self.osr) };
    }
}
