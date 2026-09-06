mod identity;
mod preferences;
mod profile;

pub use identity::PlayerIdentity;
pub use preferences::*;
pub use profile::{LocalPlayerProfile, PlayerRatingProfile};
use std::path::PathBuf;

fn config_file(name: &str) -> Option<PathBuf> {
    if let Some(directory) = std::env::var_os("LEOCARD_CONFIG_DIR") {
        return Some(PathBuf::from(directory).join(name));
    }
    #[cfg(target_os = "windows")]
    let base = std::env::var_os("APPDATA").map(PathBuf::from);
    #[cfg(target_os = "macos")]
    let base = std::env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join("Library/Application Support"));
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME")
                .map(PathBuf::from)
                .map(|home| home.join(".config"))
        });
    base.map(|base| base.join("leocard").join(name))
}
