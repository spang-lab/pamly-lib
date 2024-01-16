use anyhow::{bail, Result};
use clap::{Args, Parser, Subcommand};
use image::ImageOutputFormat;
use log::Level;
use std::{collections::HashMap, fs::File, path::PathBuf};

#[cfg(feature = "convert")]
use pamly::convert::{convert, downscale, Config, LockFile};

use pamly::types::{Diagnosis, Stain, TileLabel};
use pamly::Database;

/// Pamly Command line Interface
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    #[arg(short, long)]
    quiet: bool,
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate types files
    Types(TypesArgs),
    #[cfg(feature = "convert")]
    Convert(ConvertArgs),
    #[cfg(feature = "convert")]
    Downscale(DownscaleArgs),
    Thumbnail(ThumbnailArgs),
    Metadata(MetadataArgs),
}

#[derive(Args)]
struct MetadataArgs {
    #[arg(value_name = "Input Path")]
    path: String,
}

#[derive(Args)]
struct TypesArgs {
    #[arg(value_name = "Output Path")]
    out_path: Option<String>,
}

#[derive(Args)]
struct ThumbnailArgs {
    #[arg(value_name = "Input Path")]
    path: String,
    #[arg(value_name = "Output Path")]
    out_path: Option<String>,
    #[arg(short, long, default_value_t = 1024)]
    size: u64,
}

#[derive(Args)]
struct ConvertArgs {
    /// The path to the slide
    #[arg(value_name = "Slide Path")]
    path_str: String,
    /// Optional config file
    #[arg(short, long)]
    config: Option<String>,
    /// Output file path, default ./slide.sqlite
    #[arg(short, long)]
    output: Option<String>,
    /// Ignore overwrite of slide and existing lock
    #[arg(short, long)]
    force: bool,
}

#[derive(Args)]
struct DownscaleArgs {
    /// The path to the slide
    #[arg(value_name = "Slide Path")]
    path_str: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let mut level = Level::Info;
    if cli.quiet {
        level = Level::Warn;
    }
    if cli.verbose {
        level = Level::Debug;
    }
    simple_logger::init_with_level(level)?;

    match &cli.command {
        #[cfg(feature = "convert")]
        Commands::Convert(args) => {
            let ConvertArgs {
                path_str,
                config,
                output,
                force,
            } = args;
            let path = PathBuf::from(path_str);
            if !path.is_file() {
                log::error!("{} is not a file.", path.display());
                bail!("{} is not a file.", path.display());
            }
            let db_path = match output {
                Some(s) => PathBuf::from(s),
                None => {
                    let base_path = path.parent().unwrap();
                    base_path.join("slide.sqlite").to_owned()
                }
            };
            if db_path.is_file() {
                if *force {
                    log::warn!("Overwriting {}", db_path.display());
                    std::fs::remove_file(&db_path)?;
                } else {
                    log::error!("{} already exists", db_path.display());
                    bail!("{} already exists", db_path.display());
                }
            }
            if LockFile::exists(&db_path)? {
                if *force {
                    log::warn!("Ignoriung lockfile");
                } else {
                    log::debug!("LockFile exists. Skipping.");
                    return Ok(());
                }
            }

            let config = match config {
                Some(s) => {
                    let path = PathBuf::from(s);
                    Config::from(path)?
                }
                None => Config::default(),
            };
            log::debug!("Config:\n {}", serde_json::to_string_pretty(&config)?);
            log::debug!("Convert from {} to {}", &path.display(), &db_path.display());
            convert(path, db_path, &config)?;
        }

        #[cfg(feature = "convert")]
        Commands::Downscale(args) => {
            let DownscaleArgs { path_str } = args;
            let db_path = PathBuf::from(path_str);
            let mut lock = LockFile::lock(&db_path, "Init")?;
            let db = Database::open_readwrite(&db_path)?;
            downscale(&db, &mut lock)?;
            lock.release()?;
        }

        Commands::Thumbnail(args) => {
            let ThumbnailArgs {
                path,
                out_path,
                size,
            } = args;

            let slide_path = PathBuf::from(path);
            if !slide_path.is_file() {
                bail!("Path {} does not exist", slide_path.display());
            }
            let mut output = match out_path {
                Some(s) => PathBuf::from(s),
                None => PathBuf::from("."),
            };
            if output.is_dir() {
                output = output.join("thumbnail.jpg");
            }
            let ext = match slide_path.extension() {
                Some(oss) => oss.to_string_lossy().to_string(),
                None => "".to_owned(),
            };
            if ext == "sqlite" {
                let slide = Database::open(&slide_path)?;
                let patch = slide.thumbnail(*size)?;
                let image = patch.image()?;
                let mut out_file = File::create(output)?;
                image.write_to(&mut out_file, ImageOutputFormat::Jpeg(95))?;
                dbg!(size);
            } else {
                bail!("Only sqlite thumbnails are supported.")
            }
        }
        Commands::Metadata(args) => {
            let MetadataArgs { path } = args;
            dbg!(path);
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
