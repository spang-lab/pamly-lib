use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::File, path::PathBuf};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub tile_size: u64,
    pub edge_detect_size: u64,
    pub edge_low_threshold: f32,
    pub edge_high_threshold: f32,
    pub min_edge_content: f64,
    pub dark_threshold: f64,
    pub min_dark_content: f64,
    pub island_size: u64,
    pub island_tiles: u64,
}

impl Config {
    pub fn default() -> Config {
        log::debug!("Loading default config");
        let c = Config {
            tile_size: 512,
            edge_detect_size: 64,
            edge_low_threshold: 5.0,
            edge_high_threshold: 30.0,
            min_edge_content: 0.03,
            dark_threshold: 0.80,
            min_dark_content: 0.30,
            island_size: 5,
            island_tiles: 15,
        };
        c
    }
    pub fn from(path: PathBuf) -> Result<Config> {
        log::debug!("Reading config from {}", path.display());
        if !path.is_file() {
            bail!("Config {} is not a file", path.display());
        }
        let file = File::open(path)?;
        let c: Config = serde_json::from_reader(file)?;
        Ok(c)
    }

    pub fn to_hash_map(&self) -> Result<HashMap<String, String>> {
        let mut map = HashMap::new();
        map.insert("tile_size".to_owned(), self.tile_size.to_string());
        map.insert(
            "edge_detect_size".to_owned(),
            self.edge_detect_size.to_string(),
        );
        map.insert(
            "edge_low_threshold".to_owned(),
            self.edge_low_threshold.to_string(),
        );
        map.insert(
            "edge_high_threshold".to_owned(),
            self.edge_high_threshold.to_string(),
        );
        map.insert(
            "min_edge_content".to_owned(),
            self.min_edge_content.to_string(),
        );
        map.insert("dark_threshold".to_owned(), self.dark_threshold.to_string());
        map.insert(
            "min_dark_content".to_owned(),
            self.min_dark_content.to_string(),
        );
        map.insert("island_size".to_owned(), self.island_size.to_string());
        map.insert("island_tiles".to_owned(), self.island_tiles.to_string());
        Ok(map)
    }
}
