//! Client-only game appearance, stored independently of identity and game rules.

use super::super::config_file;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::ErrorKind;

#[derive(Serialize, Deserialize)]
pub struct GameAppearancePreferences {
    pub minecraft_guess_random_background: bool,
}

impl Default for GameAppearancePreferences {
    fn default() -> Self {
        Self {
            minecraft_guess_random_background: true,
        }
    }
}

impl GameAppearancePreferences {
    pub fn load() -> Result<Self, String> {
        let path = config_file("game-appearance.dat")
            .ok_or_else(|| "无法确定游戏外观设置目录".to_owned())?;
        match fs::read(path) {
            Ok(bytes) => {
                postcard::from_bytes(&bytes).map_err(|error| format!("游戏外观设置已损坏：{error}"))
            }
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(Self::default()),
            Err(error) => Err(format!("无法读取游戏外观设置：{error}")),
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let path = config_file("game-appearance.dat")
            .ok_or_else(|| "无法确定游戏外观设置目录".to_owned())?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("无法创建游戏外观设置目录：{error}"))?;
        }
        let bytes = postcard::to_allocvec(self)
            .map_err(|error| format!("无法编码游戏外观设置：{error}"))?;
        let temporary = path.with_extension("dat.tmp");
        fs::write(&temporary, bytes).map_err(|error| format!("无法保存游戏外观设置：{error}"))?;
        fs::rename(temporary, path).map_err(|error| format!("无法更新游戏外观设置：{error}"))
    }
}
