#[cfg(feature = "openslide")]
mod openslide;

mod types;

mod util;

use pyo3::prelude::*;

/// Formats the sum of two numbers as string.
#[pyfunction]
pub fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
    Ok((a + b).to_string())
}

/// A Python module implemented in Rust.
#[pymodule]
fn pamly(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
    m.add_class::<types::Diagnosis>()?;
    m.add_class::<types::Stain>()?;
    m.add_class::<types::TileLabel>()?;
    Ok(())
}
