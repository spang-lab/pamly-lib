#[cfg(feature = "openslide")]
mod openslide;

pub mod types;

mod util;

use pyo3::prelude::*;

/// A Python module implemented in Rust.
#[pymodule]
fn pamly(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<types::Diagnosis>()?;
    m.add_class::<types::Stain>()?;
    m.add_class::<types::TileLabel>()?;
    Ok(())
}
