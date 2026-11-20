#[cfg(not(target_arch = "wasm32"))]
use std::env;
#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;

/// Locate the workspace assets directly during development; no staging copy is needed.
pub fn asset_file_path() -> String {
    #[cfg(target_arch = "wasm32")]
    return "assets".to_owned();
    #[cfg(not(target_arch = "wasm32"))]
    native_asset_file_path()
}

#[cfg(not(target_arch = "wasm32"))]
fn native_asset_file_path() -> String {
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
