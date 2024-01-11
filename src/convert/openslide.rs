use anyhow::{bail, Result};
use libc;
use std::{self, collections::HashMap, ffi, str};

use image::{ImageBuffer, RgbImage};

use std::path::PathBuf;

const LEVEL: i32 = 0;
const X_RESOLUTION_KEY: &'static str = "openslide.mpp-x";
const Y_RESOLUTION_KEY: &'static str = "openslide.mpp-y";

enum OpenSlideT {}

#[link(name = "openslide")]
extern "C" {
    fn openslide_detect_vendor(filename: *const libc::c_char) -> *const libc::c_char;
    fn openslide_open(filename: *const libc::c_char) -> *const OpenSlideT;
    fn openslide_close(osr: *const OpenSlideT) -> libc::c_void;
    fn openslide_get_level_dimensions(
        osr: *const OpenSlideT,
        level: i32,
        w: *mut i64,
        h: *mut i64,
    ) -> libc::c_void;
    fn openslide_get_level_downsample(osr: *const OpenSlideT, level: i32) -> libc::c_double;
    fn openslide_read_region(
        osr: *const OpenSlideT,
        dest: *mut u32,
        x: i64,
        y: i64,
        level: i32,
        w: i64,
        h: i64,
    ) -> libc::c_void;
    fn openslide_get_property_names(osr: *const OpenSlideT) -> *const *const libc::c_char;
    fn openslide_get_property_value(
        osr: *const OpenSlideT,
        name: *const libc::c_char,
    ) -> *const libc::c_char;
}

/// Quickly determine whether a whole slide image is recognized.
fn detect_vendor(filename: &str) -> Result<String> {
    let c_filename = ffi::CString::new(filename)?;
    let vendor = unsafe {
        let c_vendor = openslide_detect_vendor(c_filename.as_ptr());
        if c_vendor.is_null() {
            bail!("Not a slide file");
        }
        ffi::CStr::from_ptr(c_vendor).to_string_lossy().into_owned()
    };
    Ok(vendor)
}

/// Open a whole slide image.
fn open(filename: &str) -> Result<*const OpenSlideT> {
    let c_filename = ffi::CString::new(filename)?;
    let slide = unsafe { openslide_open(c_filename.as_ptr()) };
    Ok(slide)
}

/// Close an OpenSlide object.
unsafe fn close(osr: *const OpenSlideT) {
    openslide_close(osr); // This is unsafe
}

/// Get the dimensions of a level.
unsafe fn get_level_dimensions(osr: *const OpenSlideT, level: i32) -> Result<(i64, i64)> {
    let mut width: i64 = 0;
    let mut height: i64 = 0;
    openslide_get_level_dimensions(osr, level, &mut width, &mut height); // This is unsafe
    Ok((width, height))
}

/// Get the downsampling factor of a given level.
unsafe fn get_level_downsample(osr: *const OpenSlideT, level: i32) -> Result<f64> {
    let downsampling_factor = openslide_get_level_downsample(osr, level); // This is unsafe
    Ok(downsampling_factor)
}

/// Copy pre-multiplied ARGB data from a whole slide image.
unsafe fn read_region(
    osr: *const OpenSlideT,
    x: i64,
    y: i64,
    level: i32,
    w: i64,
    h: i64,
) -> Result<Vec<u32>> {
    let mut buffer: Vec<u32> = Vec::with_capacity((h * w) as usize);
    let p_buffer = buffer.as_mut_ptr();
    openslide_read_region(osr, p_buffer, x, y, level, w, h); // This is unsafe
    buffer.set_len((h * w) as usize);
    Ok(buffer)
}

/// Get the NULL-terminated array of property names.
unsafe fn get_property_names(osr: *const OpenSlideT) -> Result<Vec<String>> {
    let string_values = {
        let null_terminated_array_ptr = openslide_get_property_names(osr);
        let mut counter = 0;
        let mut loc = null_terminated_array_ptr;
        while !(*loc).is_null() {
            counter += 1;
            loc = loc.offset(1);
        }
        //let c_array = ffi::CStr::from_ptr(null_terminated_array_ptr);
        let values = std::slice::from_raw_parts(null_terminated_array_ptr, counter as usize);
        values
            .iter()
            .map(|&p| ffi::CStr::from_ptr(p)) // iterator of &CStr
            .map(|cs| cs.to_bytes()) // iterator of &[u8]
            .map(|bs| str::from_utf8(bs).unwrap()) // iterator of &str
            .map(|ss| ss.to_owned())
            .collect()
    };
    Ok(string_values)
}

/// Get the value of a single property.
unsafe fn get_property_value(osr: *const OpenSlideT, name: &str) -> Result<String> {
    let c_name = ffi::CString::new(name)?;
    let value = {
        let c_value = openslide_get_property_value(osr, c_name.as_ptr());
        ffi::CStr::from_ptr(c_value).to_string_lossy().into_owned()
    };
    Ok(value)
}

#[derive(Debug, Clone)]
pub struct OpenSlide {
    osr: *const OpenSlideT,
    pub width: u64,
    pub height: u64,
    pub downsample: f64,
    pub vendor: String,
}
unsafe impl Send for OpenSlide {}
unsafe impl Sync for OpenSlide {}

impl OpenSlide {
    pub fn open(path: &PathBuf) -> Result<OpenSlide> {
        let raw_path = match path.to_str() {
            Some(p) => p,
            None => bail!("Path utf8 coding error"),
        };
        let vendor = match detect_vendor(raw_path) {
            Ok(vendor) => vendor,
            Err(_) => "Unknown".to_owned(),
        };
        let osr = match open(raw_path) {
            Ok(osr) => osr,
            Err(_) => bail!("Failed to open slide"),
        };
        let downsample = match unsafe { get_level_downsample(osr, LEVEL) } {
            Ok(f) => f,
            Err(_) => bail!("Failed to get downsample factor"),
        };
        let (width, height) = match unsafe { get_level_dimensions(osr, LEVEL) } {
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
        let data = match unsafe { read_region(self.osr, x, y, LEVEL, width, height) } {
            Ok(r) => r,
            Err(_) => bail!("Call to read_region failed"),
        };
        let mut rgb: Vec<u8> = Vec::with_capacity(data.len() * 4);
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

    pub fn get_metadata(&self) -> Result<HashMap<String, String>> {
        let mut data = HashMap::new();
        let names = match unsafe { get_property_names(self.osr) } {
            Ok(n) => n,
            Err(_) => bail!("Failed to read property names."),
        };
        for name in names {
            let value = match unsafe { get_property_value(self.osr, &name) } {
                Ok(v) => v,
                Err(_) => bail!("Failed to get_property_value {}", name),
            };
            data.insert(name, value);
        }
        return Ok(data);
    }

    // returns slide resolution in pixels per meter
    pub fn get_resolution(&self) -> Result<(u64, u64)> {
        let names = match unsafe { get_property_names(self.osr) } {
            Ok(n) => n,
            Err(_) => bail!("Failed to read property names."),
        };
        let x_key = X_RESOLUTION_KEY.to_owned();
        let y_key = Y_RESOLUTION_KEY.to_owned();
        if !names.contains(&x_key) || !names.contains(&y_key) {
            bail!("Could not read resolution from slide");
        }
        let x_res_str = match unsafe { get_property_value(self.osr, &x_key) } {
            Ok(x) => x,
            Err(_) => bail!("Failed to read property value {}", &x_key),
        };
        let y_res_str = match unsafe { get_property_value(self.osr, &y_key) } {
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
        unsafe { close(self.osr) };
    }
}
