use super::mode::PlayMode;
use crate::player::config_file;
use leocard_music::SavedSession;
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{ErrorKind, Write};
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;

#[derive(Deserialize, Serialize)]
#[serde(default)]
pub(super) struct MusicPreferences {
    pub volume: f32,
    pub session: Option<SavedSession>,
    pub mode: PlayMode,
    pub show_in_game: bool,
}

impl Default for MusicPreferences {
    fn default() -> Self {
        Self {
            volume: 0.5,
            session: None,
            mode: PlayMode::default(),
            show_in_game: true,
        }
    }
}

impl MusicPreferences {
    pub fn load() -> Result<Self, String> {
        let path = config_file("music.json").ok_or("无法确定音乐配置目录")?;
        let bytes = match fs::read(path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Self::default()),
            Err(_) => return Err("无法读取音乐配置".to_owned()),
        };
        let mut preferences: Self =
            serde_json::from_slice(&bytes).map_err(|_| "音乐配置无效，请重新关联账号")?;
        if !preferences.volume.is_finite() {
            preferences.volume = 0.5;
        }
        preferences.volume = preferences.volume.clamp(0.0, 1.0);
        Ok(preferences)
    }

    pub fn save(&self) -> Result<(), String> {
        let path = config_file("music.json").ok_or("无法确定音乐配置目录")?;
        fs::create_dir_all(path.parent().ok_or("无法确定音乐配置目录")?)
            .map_err(|_| "无法创建音乐配置目录")?;
        let temporary = path.with_extension("json.tmp");
        let mut options = OpenOptions::new();
        options.write(true).create(true).truncate(true);
        #[cfg(unix)]
        options.mode(0o600);
        // Keep music credentials separate from game preferences and profile exports.
        let mut file = options.open(&temporary).map_err(|_| "无法保存音乐配置")?;
        let bytes = serde_json::to_vec(self).map_err(|_| "音乐配置编码失败")?;
        file.write_all(&bytes)
            .and_then(|()| file.sync_all())
            .map_err(|_| "无法保存音乐配置")?;
        drop(file);
        fs::rename(temporary, path).map_err(|_| "无法更新音乐配置".to_owned())
    }
}
