use std::fs::read_dir;
use std::path::PathBuf;

use anyhow::{bail, Result};

use super::convert;
use super::LockFile;

use super::{Config, OpenSlide};

pub fn convert_all(
    input_path: PathBuf,
    output_path: PathBuf,
    config: &Config,
    force: bool,
) -> Result<()> {
    for entry in read_dir(&input_path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            match OpenSlide::open(&path) {
                Ok(_) => {
                    let mut output_file = match path.file_stem() {
                        Some(s) => output_path.join(s),
                        None => bail!("Invalid filename"),
                    };
                    output_file.set_extension("sqlite");
                    if output_file.is_file() {
                        if force {
                            log::warn!("Overwriting {}", output_file.display());
                            std::fs::remove_file(&output_file)?;
                        } else {
                            log::error!("{} already exists", output_file.display());
                            bail!("{} already exists", output_file.display());
                        }
                    }
                    if LockFile::exists(&output_file)? {
                        if force {
                            log::warn!("Ignoring lockfile");
                        } else {
                            log::debug!("LockFile exists. Skipping.");
                            return Ok(());
                        }
                    }
                    log::debug!("Converting {} to {}", path.display(), output_file.display());
                    convert(path.clone(), output_file, config)?;
                }
                Err(_) => {
                    log::debug!("{} is not a slide file. Ignoring.", path.display());
                }
            }
        }
        if path.is_dir() {
            let relative_path = path.strip_prefix(&input_path)?;
            let new_dir = output_path.join(relative_path);
            std::fs::create_dir_all(&new_dir)?;
            convert_all(path, new_dir, config, force)?;
        }
    }
    Ok(())
}
