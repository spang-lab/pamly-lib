#[cfg(feature = "convert")]
mod convert;

pub mod types;
pub use types::*;
mod util;

use pyo3::prelude::*;

#[pymodule]
fn pamly(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<types::Diagnosis>()?;
    m.add_class::<types::Stain>()?;
    m.add_class::<types::TileLabel>()?;
    Ok(())
}
