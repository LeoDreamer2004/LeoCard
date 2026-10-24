use leocard_protocol::PlayerGender;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const DEFAULT_AUDIO_VOLUME: f32 = 0.8;
const DEFAULT_TABLE_VIGNETTE: f32 = 0.38;

#[derive(Deserialize, Serialize)]
pub struct GlobalPreferences {
    pub player_name: String,
    pub avatar_png: Option<Vec<u8>>,
    pub host_port: String,
    pub join_address: String,
    pub table_felt_path: Option<PathBuf>,
    pub table_brightness: f32,
    pub table_vignette: f32,
    pub audio_volume: f32,
    pub gender: PlayerGender,
    pub mahjong_action_voices: bool,
    pub mahjong_fan_voices: bool,
    pub texas_action_voices: bool,
}

impl Default for GlobalPreferences {
    fn default() -> Self {
        Self {
            player_name: String::new(),
            avatar_png: None,
            host_port: String::new(),
            join_address: String::new(),
            table_felt_path: None,
            table_brightness: 1.0,
            table_vignette: DEFAULT_TABLE_VIGNETTE,
            audio_volume: DEFAULT_AUDIO_VOLUME,
            gender: PlayerGender::Male,
            mahjong_action_voices: true,
            mahjong_fan_voices: true,
            texas_action_voices: true,
        }
    }
}
