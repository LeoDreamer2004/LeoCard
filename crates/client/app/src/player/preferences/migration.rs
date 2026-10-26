use super::files::PreferenceFiles;
use super::{GamePreferences, GlobalPreferences, SavedPreferences};
use leocard_protocol::PlayerGender;
use serde::{Deserialize, de::DeserializeOwned};
use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;

#[derive(Deserialize)]
struct LegacyGlobalPreferences {
    player_name: String,
    avatar_png: Option<Vec<u8>>,
    host_port: String,
    join_address: String,
    table_felt_path: Option<PathBuf>,
    table_brightness: f32,
    table_vignette: f32,
    audio_volume: f32,
    gender: PlayerGender,
    mahjong_action_voices: bool,
    mahjong_fan_voices: bool,
    texas_action_voices: bool,
}

impl From<LegacyGlobalPreferences> for GlobalPreferences {
    fn from(previous: LegacyGlobalPreferences) -> Self {
        Self {
            player_name: previous.player_name,
            avatar_png: previous.avatar_png,
            host_port: previous.host_port,
            join_address: previous.join_address,
            table_felt_path: previous.table_felt_path,
            table_brightness: previous.table_brightness,
            table_vignette: previous.table_vignette,
            audio_volume: previous.audio_volume,
            gender: previous.gender,
            mahjong_action_voices: previous.mahjong_action_voices,
            mahjong_fan_voices: previous.mahjong_fan_voices,
            texas_action_voices: previous.texas_action_voices,
        }
    }
}

// Consume the old host_rules layout only to reach the following game settings.
type V4TexasPreferences = ((u8, u16, u8, bool, bool, bool, bool),);

fn decode_games(bytes: &[u8], version: u8) -> Option<GamePreferences> {
    if version == 5 {
        return decode_exact(bytes);
    }
    let (qigui523, _, shengji, uno, mahjong, minecraft_guess) =
        decode_exact::<(_, V4TexasPreferences, _, _, _, _)>(bytes)?;
    Some(GamePreferences {
        qigui523,
        texas_holdem: GamePreferences::default().texas_holdem,
        shengji,
        uno,
        mahjong,
        minecraft_guess,
    })
}

fn decode_exact<T: DeserializeOwned>(bytes: &[u8]) -> Option<T> {
    let (value, remaining) = postcard::take_from_bytes(bytes).ok()?;
    remaining.is_empty().then_some(value)
}

pub(super) fn migrate(files: &PreferenceFiles) -> Result<(), String> {
    let source = files.legacy_path();
    let bytes = match fs::read(&source) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(format!("无法读取旧偏好：{error}")),
    };
    let (version, payload) = if let Some(payload) = bytes.strip_prefix(b"LeoCard/prefs/v4\0") {
        (4, payload)
    } else if let Some(payload) = bytes.strip_prefix(b"LeoCard/prefs/v5\0") {
        (5, payload)
    } else {
        return Err("无法识别旧偏好格式，原文件已保留".to_owned());
    };
    // Decode global settings first so a rule-layout failure cannot discard avatars.
    let (global, games) = postcard::take_from_bytes::<LegacyGlobalPreferences>(payload)
        .map_err(|error| format!("无法迁移全局偏好，原文件已保留：{error}"))?;
    files.import(&SavedPreferences {
        global: global.into(),
        games: decode_games(games, version).unwrap_or_default(),
    })?;
    files.load()?;
    // Interrupted imports resume missing files; archive the source only after every write succeeds.
    fs::rename(&source, source.with_extension("prefs.migrated"))
        .map_err(|error| format!("无法完成偏好迁移，原文件已保留：{error}"))
}
