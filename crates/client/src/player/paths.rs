#[cfg(target_os = "android")]
use crate::platform::data_directory;
#[cfg(not(target_os = "android"))]
use std::env;
use std::path::PathBuf;

pub(super) fn config_file(name: &str) -> Option<PathBuf> {
    #[cfg(target_os = "android")]
    return data_directory().map(|directory| directory.join(name));

    #[cfg(not(target_os = "android"))]
    desktop_config_file(name)
}

#[cfg(not(target_os = "android"))]
fn desktop_config_file(name: &str) -> Option<PathBuf> {
    if let Some(directory) = env::var_os("LEOCARD_CONFIG_DIR") {
        return Some(PathBuf::from(directory).join(name));
    }
    #[cfg(target_os = "windows")]
    let base = env::var_os("APPDATA").map(PathBuf::from);
    #[cfg(target_os = "macos")]
    let base = env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join("Library/Application Support"));
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let base = env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            env::var_os("HOME")
                .map(PathBuf::from)
                .map(|home| home.join(".config"))
        });
    base.map(|base| base.join("leocard").join(name))
}
