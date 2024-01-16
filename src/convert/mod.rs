mod config;
pub use config::Config;

mod openslide;
pub use openslide::OpenSlide;

mod actions;
pub use actions::*;

mod convert;
pub use convert::convert;
