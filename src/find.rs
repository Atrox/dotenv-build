use std::fs::File;
use std::path::{Path, PathBuf};
use std::{env, fs, io};

use crate::errors::*;
use crate::iter::Iter;
use crate::Config;

pub fn find(config: &Config) -> Result<(PathBuf, Iter<File>)> {
    let path = find_internal(
        &env::current_dir().map_err(Error::Io)?,
        config.filename,
        config.recursive_search,
    )?;
    let file = File::open(&path).map_err(Error::Io)?;
    let iter = Iter::new(file);
    Ok((path, iter))
}

fn find_internal(directory: &Path, filename: &Path, recursive: bool) -> Result<PathBuf> {
    let candidate = directory.join(filename);

    match fs::metadata(&candidate) {
        Ok(metadata) => {
            if metadata.is_file() {
                return Ok(candidate);
            }
        }
        Err(error) => {
            if error.kind() != io::ErrorKind::NotFound {
                return Err(Error::Io(error));
            }
        }
    }

    match directory.parent() {
        Some(parent) if recursive => find_internal(parent, filename, recursive),
        _ => Err(Error::Io(io::Error::new(
            io::ErrorKind::NotFound,
            "path not found",
        ))),
    }
}
