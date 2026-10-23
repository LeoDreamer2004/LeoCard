use std::env;
use std::path::Path;

/// Locate the workspace assets directly during development; no staging copy is needed.
pub fn asset_file_path() -> String {
    if env::var_os("BEVY_ASSET_ROOT").is_some() {
        return "assets".to_owned();
    }
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../assets");
    if workspace.is_dir() {
        workspace.to_string_lossy().into_owned()
    } else {
        "assets".to_owned()
    }
}
