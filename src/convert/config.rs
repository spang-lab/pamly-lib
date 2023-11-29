use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub tile_size: u64,
    pub edge_detect_size: u64,
    pub edge_low_threshold: f32,
    pub edge_high_threshold: f32,
    pub min_edge_content: f64,
    pub dark_threshold: f64,
    pub min_dark_content: f64,
    pub min_island_size: u64,
    pub slide_file: String,
    pub state_file: String,
}

impl Config {
    pub fn default() -> Config {
        Config {
            tile_size: 521,
            edge_detect_size: 64,
            edge_low_threshold: 5.0,
            edge_high_threshold: 30.0,
            min_edge_content: 0.03,
            dark_threshold: 0.80,
            min_dark_content: 0.30,
            min_island_size: 7,
            slide_file: "slide.sqlite".to_owned(),
            state_file: "state.json".to_owned(),
        }
    }
}
