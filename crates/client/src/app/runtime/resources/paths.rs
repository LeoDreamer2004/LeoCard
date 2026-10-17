//! 运行时资源根目录。

use std::env;

pub(crate) fn asset_file_path() -> String {
    if env::var_os("BEVY_ASSET_ROOT").is_some() {
        return "assets".to_owned();
    }
    if env::var_os("CARGO_MANIFEST_DIR").is_some() {
        return "../../assets".to_owned();
    }
    if env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(ToOwned::to_owned))
        .is_some_and(|directory| directory.join("../../assets").is_dir())
    {
        "../../assets".to_owned()
    } else {
        "assets".to_owned()
    }
}
