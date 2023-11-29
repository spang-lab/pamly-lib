use anyhow::{bail, Result};

use clap::{Arg, ArgAction, Command};

use std::fs;
use std::path::PathBuf;

mod convert;
use convert::convert_slide;

mod lock;
use lock::Lock;

mod config;
use config::Config;

mod islands;
use islands::delete_islands;

fn main() -> Result<()> {
    let matches = Command::new("converter")
        .version("1.0.0")
        .author("Michael Huttner <michael@mhuttner.com>")
        .about("Converts ndpi files to sqlite")
        .arg(
            Arg::new("overwrite")
                .help("Overwrite existing files")
                .short('f')
                .long("overwrite")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("ignore-lock")
                .help("Ingore Lock file")
                .short('l')
                .long("ignore-lock")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("quiet")
                .help("do not print progress")
                .short('q')
                .long("quiet")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("tile-size")
                .help("Set the tile size")
                .short('s')
                .long("tile-size")
                .default_value("512"),
        )
        .arg(
            Arg::new("edge-detect-size")
                .help("image size for edge detection")
                .long("edge-detect-size")
                .default_value("64"),
        )
        .arg(
            Arg::new("edge-low-threshold")
                .help("lower threshold for edge detection")
                .long("edge-low-threshold")
                .default_value("5.0"),
        )
        .arg(
            Arg::new("edge-high-threshold")
                .help("upper threshold for edge detection")
                .long("edge-high-threshold")
                .default_value("30.0"),
        )
        .arg(
            Arg::new("min-edge-content")
                .help("minimum amount of edges")
                .long("min-edge-content")
                .default_value("0.03"),
        )
        .arg(
            Arg::new("dark-threshold")
                .help("pixel grayscale color to classified as dark from (0, 1)")
                .long("dark-threshold")
                .default_value("0.80"),
        )
        .arg(
            Arg::new("min-dark-content")
                .help("minimum amount of dark pixels")
                .long("min-dark-content")
                .default_value("0.30"),
        )
        .arg(
            Arg::new("min-island-size")
                .help("minimum size of an island of tiles")
                .long("min-island-size")
                .default_value("7"),
        )
        .arg(
            Arg::new("input")
                .help("the input path")
                .index(1)
                .required(true),
        )
        .get_matches();

    if matches.get_flag("overwrite") {
        println!("Overwriting existing files.");
    }
    let input_path = PathBuf::from(matches.get_one::<String>("input").unwrap());

    let config = Config::from(&matches)?;

    if input_path.is_file() {
        bail!(
            "Expected folder as input path, got {}",
            input_path.to_string_lossy()
        )
    }
    let lock_file = "converter.lock";
    let mut folders = Vec::new();
    for entry in fs::read_dir(input_path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            folders.push(path);
        }
    }
    let mut slide_files = Vec::new();
    for folder in folders {
        let mut candidates = Vec::new();
        for entry in fs::read_dir(folder.clone())? {
            let path = entry?.path();
            if !path.is_file() {
                continue;
            }
            let ext = path.extension();
            if ext.is_none() {
                continue;
            }
            let ext = ext.unwrap();
            if ext == "sqlite" || ext == "lock" {
                continue;
            }
            candidates.push(path);
        }
        if candidates.len() == 0 {
            println!("No slide file found in {}", folder.to_string_lossy());
            continue;
        }
        if candidates.len() > 1 {
            println!(
                "Multiple possible slides found in {}",
                folder.to_string_lossy()
            );
            continue;
        }
        slide_files.push(candidates.pop().unwrap());
    }

    for path in slide_files {
        let base_path = path.parent().unwrap();
        let sqlite_path = base_path.join("slide.sqlite").to_owned();
        let lock_path = base_path.join(lock_file);
        let quiet = matches.get_flag("quiet");
        let mut lock = Lock::new(lock_path, quiet);

        if lock.exists() {
            if !matches.get_flag("ignore-lock") {
                continue;
            } else {
                println!("Ingoring existing lock file");
            }
        }

        if sqlite_path.exists() && sqlite_path.is_file() {
            if matches.get_flag("overwrite") {
                println!("Overwriting {:?}", sqlite_path);
                fs::remove_file(&sqlite_path)?;
            } else {
                continue;
            }
        }
        lock.create()?;
        match convert_slide(path.clone(), sqlite_path.clone(), &mut lock, &config) {
            Ok(_) => {
                println!("Finished slide {:?}", sqlite_path);
                lock.delete()?;
            }
            Err(e) => {
                println!("Error converting slide {:?}: {}", path, e);
                lock.error(e)?;
            }
        };
    }
    Ok(())
}
