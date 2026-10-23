//! Local spectator switches are separate from the versioned game/rule preferences.

use super::config_file;
use std::{fs, io::ErrorKind};

#[derive(Default)]
pub struct TexasSpectatorPreferences {
    pub show_win_rates: bool,
}

impl TexasSpectatorPreferences {
    pub fn load() -> Result<Self, String> {
        let path =
            config_file("texas-spectator.dat").ok_or_else(|| "无法确定旁观设置目录".to_owned())?;
        match fs::read(path) {
            Ok(bytes) => match bytes.as_slice() {
                [0] => Ok(Self::default()),
                [1] => Ok(Self {
                    show_win_rates: true,
                }),
                _ => Err("德州旁观设置已损坏".to_owned()),
            },
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(Self::default()),
            Err(error) => Err(format!("无法读取德州旁观设置：{error}")),
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let path =
            config_file("texas-spectator.dat").ok_or_else(|| "无法确定旁观设置目录".to_owned())?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| format!("无法创建旁观设置目录：{error}"))?;
        }
        let temporary = path.with_extension("dat.tmp");
        fs::write(&temporary, [u8::from(self.show_win_rates)])
            .map_err(|error| format!("无法保存旁观设置：{error}"))?;
        fs::rename(temporary, path).map_err(|error| format!("无法更新旁观设置：{error}"))
    }
}
