use std::{collections::HashMap, fs::File, path::PathBuf};

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
    #[cfg(feature = "convert")]
    Convert(ConvertArgs),
}

#[derive(Args)]
struct TypesArgs {
    #[arg(value_name = "Output Path")]
    out_path: Option<String>,
}

#[derive(Args)]
struct ConvertArgs {
    /// The path to the slide
    #[arg(value_name = "Slide Path")]
    path_str: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        #[cfg(feature = "convert")]
        Commands::Convert(args) => {
            let ConvertArgs { path_str } = args;
            let path = PathBuf::from(path_str);
            let base_path = path.parent().unwrap();
            let sqlite_path = base_path.join("slide.sqlite").to_owned();
            dbg!(path, sqlite_path);
        }
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
