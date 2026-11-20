use std::fs::{self, OpenOptions};
use std::io::{self, Write};
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

pub use std::fs::{create_dir_all, read, remove_file, rename, write};

pub fn is_file(path: impl AsRef<Path>) -> bool {
    path.as_ref().is_file()
}

pub fn try_exists(path: impl AsRef<Path>) -> io::Result<bool> {
    path.as_ref().try_exists()
}

pub fn list_files(path: impl AsRef<Path>) -> io::Result<Vec<PathBuf>> {
    fs::read_dir(path)?
        .filter_map(|entry| match entry {
            Ok(entry) => match entry.file_type() {
                Ok(kind) if kind.is_file() => Some(Ok(entry.path())),
                Ok(_) => None,
                Err(error) => Some(Err(error)),
            },
            Err(error) => Some(Err(error)),
        })
        .collect()
}

/// Replace a complete archive atomically and restrict newly written files to their owner.
pub fn write_atomic(path: impl AsRef<Path>, bytes: impl AsRef<[u8]>) -> io::Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let temporary = path.with_extension(format!("{:016x}.tmp", fastrand::u64(..)));
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    options.mode(0o600);
    let result = (|| {
        let mut file = options.open(&temporary)?;
        file.write_all(bytes.as_ref())?;
        file.sync_all()?;
        drop(file);
        fs::rename(&temporary, path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(temporary);
    }
    result
}
