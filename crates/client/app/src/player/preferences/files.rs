use super::{GamePreferences, GlobalPreferences, SavedPreferences};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;

pub(super) struct PreferenceFiles {
    directory: PathBuf,
}

macro_rules! game_files {
    ($($field:ident => $file:literal),+ $(,)?) => {
        fn load_games(&self) -> GamePreferences {
            let defaults = GamePreferences::default();
            GamePreferences {
                $($field: self.read($file).ok().flatten().unwrap_or(defaults.$field),)+
            }
        }

        pub(super) fn save_games(&self, preferences: &GamePreferences) -> Result<(), String> {
            $(self.write($file, &preferences.$field)?;)+
            Ok(())
        }

        fn import_games(&self, preferences: &GamePreferences) -> Result<(), String> {
            $(self.write_missing($file, &preferences.$field)?;)+
            Ok(())
        }
    };
}

impl PreferenceFiles {
    pub(super) fn new(directory: PathBuf) -> Self {
        Self { directory }
    }

    pub(super) fn legacy_path(&self) -> PathBuf {
        self.directory.with_file_name("client.prefs")
    }

    pub(super) fn load(&self) -> Result<SavedPreferences, String> {
        Ok(SavedPreferences {
            global: self.read("global.json")?.unwrap_or_default(),
            games: self.load_games(),
        })
    }

    pub(super) fn save_global(&self, preferences: &GlobalPreferences) -> Result<(), String> {
        self.write("global.json", preferences)
    }

    pub(super) fn import(&self, preferences: &SavedPreferences) -> Result<(), String> {
        self.write_missing("global.json", &preferences.global)?;
        self.import_games(&preferences.games)
    }

    game_files! {
        qigui523 => "games/qigui523.json",
        texas => "games/texas_holdem.json",
        shengji => "games/shengji.json",
        uno => "games/uno.json",
        mahjong => "games/mahjong.json",
        mc_guess => "games/mc_guess.json",
        doudizhu => "games/doudizhu.json",
    }

    fn read<T: DeserializeOwned>(&self, name: &str) -> Result<Option<T>, String> {
        let path = self.directory.join(name);
        let bytes = match fs::read(&path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(format!("无法读取偏好 {}：{error}", path.display())),
        };
        serde_json::from_slice(&bytes)
            .map(Some)
            .map_err(|error| format!("无法解析偏好 {}：{error}", path.display()))
    }

    fn write_missing<T: Serialize>(&self, name: &str, preferences: &T) -> Result<(), String> {
        let path = self.directory.join(name);
        if path.try_exists().map_err(|error| error.to_string())? {
            return if path.is_file() {
                Ok(())
            } else {
                Err(format!("偏好路径不是文件：{}", path.display()))
            };
        }
        self.write(name, preferences)
    }

    fn write<T: Serialize>(&self, name: &str, preferences: &T) -> Result<(), String> {
        let path = self.directory.join(name);
        let parent = path.parent().ok_or("无法确定偏好文件目录")?;
        fs::create_dir_all(parent).map_err(|error| format!("无法创建偏好目录：{error}"))?;
        let bytes = serde_json::to_vec_pretty(preferences)
            .map_err(|error| format!("偏好编码失败：{error}"))?;
        let temporary = path.with_extension("json.tmp");
        fs::write(&temporary, bytes).map_err(|error| format!("无法保存偏好：{error}"))?;
        fs::rename(temporary, &path)
            .map_err(|error| format!("无法更新偏好 {}：{error}", path.display()))
    }
}
