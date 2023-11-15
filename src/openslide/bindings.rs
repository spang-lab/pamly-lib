use anyhow::{bail, Error};
use libc;
use std::{self, ffi, str};

/// Dummy type for the openslide_t type in OpenSlide
pub enum OpenSlideT {}

#[link(name = "openslide")]
extern "C" {

    // ---------------
    // Basic usage
    // ---------------

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

    // ---------------
    // Error handling
    // ---------------

    // fn openslide_get_error(
    //     osr: *const OpenSlideT
    // ) -> *const libc::c_char;

    // ---------------
    // Properties
    // ---------------

    fn openslide_get_property_names(osr: *const OpenSlideT) -> *const *const libc::c_char;

    fn openslide_get_property_value(
        osr: *const OpenSlideT,
        name: *const libc::c_char,
    ) -> *const libc::c_char;
}

// ---------------
// Basic usage
// ---------------

/// Quickly determine whether a whole slide image is recognized.
pub fn detect_vendor(filename: &str) -> Result<String, Error> {
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
pub fn open(filename: &str) -> Result<*const OpenSlideT, Error> {
    let c_filename = ffi::CString::new(filename)?;
    let slide = unsafe { openslide_open(c_filename.as_ptr()) };
    Ok(slide)
}

/// Close an OpenSlide object.
pub unsafe fn close(osr: *const OpenSlideT) {
    openslide_close(osr); // This is unsafe
}

/// Get the dimensions of a level.
pub unsafe fn get_level_dimensions(
    osr: *const OpenSlideT,
    level: i32,
) -> Result<(i64, i64), Error> {
    let mut width: i64 = 0;
    let mut height: i64 = 0;
    openslide_get_level_dimensions(osr, level, &mut width, &mut height); // This is unsafe
    Ok((width, height))
}

/// Get the downsampling factor of a given level.
pub unsafe fn get_level_downsample(osr: *const OpenSlideT, level: i32) -> Result<f64, Error> {
    let downsampling_factor = openslide_get_level_downsample(osr, level); // This is unsafe
    Ok(downsampling_factor)
}

/// Copy pre-multiplied ARGB data from a whole slide image.
pub unsafe fn read_region(
    osr: *const OpenSlideT,
    x: i64,
    y: i64,
    level: i32,
    w: i64,
    h: i64,
) -> Result<Vec<u32>, Error> {
    let mut buffer: Vec<u32> = Vec::with_capacity((h * w) as usize);
    let p_buffer = buffer.as_mut_ptr();
    openslide_read_region(osr, p_buffer, x, y, level, w, h); // This is unsafe
    buffer.set_len((h * w) as usize);
    Ok(buffer)
}

// ---------------
// Properties
// ---------------

/// Get the NULL-terminated array of property names.
pub unsafe fn get_property_names(osr: *const OpenSlideT) -> Result<Vec<String>, Error> {
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
pub unsafe fn get_property_value(osr: *const OpenSlideT, name: &str) -> Result<String, Error> {
    let c_name = ffi::CString::new(name)?;
    let value = {
        let c_value = openslide_get_property_value(osr, c_name.as_ptr());
        ffi::CStr::from_ptr(c_value).to_string_lossy().into_owned()
    };
    Ok(value)
}
