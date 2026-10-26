use crate::{Result, slashes};
use std::fs;
use std::io::{self, ErrorKind};
use std::path::Path;

// std's fs, but every error names its file
pub fn read(path: &Path) -> Result<Option<String>> {
    match fs::read_to_string(path) {
        Ok(text) => Ok(Some(text)),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(failed(path, &error)),
    }
}

pub fn write(path: &Path, contents: impl AsRef<[u8]>) -> Result {
    fs::write(path, contents).map_err(|error| failed(path, &error))
}

pub fn write_new(path: &Path, contents: &str) -> Result {
    if !path.exists() {
        write(path, contents)?;
    }
    Ok(())
}

pub fn create_dir(path: &Path) -> Result {
    fs::create_dir_all(path).map_err(|error| failed(path, &error))
}

pub fn remove(path: &Path) -> Result {
    fs::remove_file(path).map_err(|error| failed(path, &error))
}

fn failed(path: &Path, error: &io::Error) -> Box<dyn std::error::Error> {
    format!("{}: {error}", slashes(path)).into()
}
