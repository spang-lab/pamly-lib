#[cfg(feature = "convert")]
mod convert;

mod database;
pub use database::Database;

mod actions;
mod util;

pub mod types;
pub use types::*;

use pyo3::prelude::*;

#[pymodule]
fn pamly(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<types::Diagnosis>()?;
    m.add_class::<types::Stain>()?;
    m.add_class::<types::TileLabel>()?;
    Ok(())
}
