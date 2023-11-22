#[cfg(feature = "openslide")]
mod openslide;

mod types;

mod util;

use neon::prelude::*;
use pyo3::prelude::*;

/// A Python module implemented in Rust.
#[pymodule]
fn pamly(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<types::Diagnosis>()?;
    m.add_class::<types::Stain>()?;
    m.add_class::<types::TileLabel>()?;
    Ok(())
}

fn hello(mut cx: FunctionContext) -> JsResult<JsString> {
    Ok(cx.string("hello node"))
}

#[neon::main]
fn neon(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("hello", hello)?;
    let tile_label = types::TileLabel::to_object(&mut cx)?;
    cx.export_value("TileLabel", tile_label)?;
    let stain = types::Stain::to_object(&mut cx)?;
    cx.export_value("Stain", stain)?;
    let diagnosis = types::Diagnosis::to_object(&mut cx)?;
    cx.export_value("Diagnosis", diagnosis)?;
    Ok(())
}
