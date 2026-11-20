use crate::browser;
use std::io;
use std::path::{Path, PathBuf};

pub fn read(path: impl AsRef<Path>) -> io::Result<Vec<u8>> {
    browser::read_file(&path.as_ref().to_string_lossy())
}

pub fn write(path: impl AsRef<Path>, bytes: impl AsRef<[u8]>) -> io::Result<()> {
    browser::write_file(&path.as_ref().to_string_lossy(), bytes.as_ref())
}

pub use write as write_atomic;

pub fn rename(from: impl AsRef<Path>, to: impl AsRef<Path>) -> io::Result<()> {
    browser::rename_file(
        &from.as_ref().to_string_lossy(),
        &to.as_ref().to_string_lossy(),
    )
}

pub fn remove_file(path: impl AsRef<Path>) -> io::Result<()> {
    browser::remove_file(&path.as_ref().to_string_lossy())
}

pub fn list_files(path: impl AsRef<Path>) -> io::Result<Vec<PathBuf>> {
    browser::list_files(&path.as_ref().to_string_lossy())
}

pub fn is_file(path: impl AsRef<Path>) -> bool {
    browser::file_exists(&path.as_ref().to_string_lossy())
}

pub fn try_exists(path: impl AsRef<Path>) -> io::Result<bool> {
    Ok(is_file(path.as_ref()) || !list_files(path)?.is_empty())
}

pub fn create_dir_all(_path: impl AsRef<Path>) -> io::Result<()> {
    Ok(())
}
