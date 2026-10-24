use super::SavedPreferences;
use crate::player::config_file;
use std::fs;

const PREFERENCES_HEADER: &[u8] = b"LeoCard/prefs/v4\0";

pub fn load_player_preferences() -> Option<SavedPreferences> {
    let bytes = fs::read(config_file("client.prefs")?).ok()?;
    let current = bytes.strip_prefix(PREFERENCES_HEADER)?;
    postcard::from_bytes(current).ok()
}

pub fn save_player_preferences(preferences: &SavedPreferences) -> Result<(), String> {
    let path = config_file("client.prefs").ok_or_else(|| "无法确定本机配置目录".to_owned())?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| format!("无法创建配置目录：{error}"))?;
    }
    let mut bytes = PREFERENCES_HEADER.to_vec();
    bytes.extend(
        postcard::to_allocvec(preferences).map_err(|error| format!("配置编码失败：{error}"))?,
    );
    fs::write(path, bytes).map_err(|error| format!("无法保存本地配置：{error}"))
}
