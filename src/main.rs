use std::{collections::HashMap, fs::File};

use anyhow::{bail, Result};
use clap::{Args, CommandFactory, Parser, Subcommand};

use pamly::types::{Diagnosis, Stain, TileLabel};

/// Pamly Command line Interface
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate types files
    Types(TypesArgs),
}

#[derive(Args)]
struct TypesArgs {
    /// The path where to new key should be stored
    #[arg(value_name = "Output Path")]
    out_path: Option<String>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Types(args) => {
            let TypesArgs { out_path } = args;

            let mut types = HashMap::new();
            let diagnosis = Diagnosis::to_hash_map();
            types.insert("diagnosis", diagnosis);
            let stain = Stain::to_hash_map();
            types.insert("stain", stain);
            let tile_label = TileLabel::to_hash_map();
            types.insert("tileLabel", tile_label);
            let path = match out_path {
                Some(p) => p.clone(),
                None => "data/types.json".to_owned(),
            };

            let file = File::create(path)?;
            serde_json::to_writer_pretty(&file, &types)?;
        }
    };
    Ok(())
}
