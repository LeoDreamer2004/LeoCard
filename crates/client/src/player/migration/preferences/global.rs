use crate::player::preferences::GlobalPreferences;
use leocard_protocol::PlayerGender;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// Postcard 顺序编码结构体字段；嵌套结构不添加边界，保留旧存档字段顺序。
#[derive(Deserialize, Serialize)]
pub(super) struct Base {
    pub player_name: String,
    pub avatar_png: Option<Vec<u8>>,
    pub host_port: String,
    pub join_address: String,
    pub table_felt_path: Option<PathBuf>,
    pub table_brightness: f32,
    pub table_vignette: f32,
    #[serde(default = "default_volume")]
    pub audio_volume: f32,
}

fn default_volume() -> f32 {
    0.8
}

impl From<Base> for GlobalPreferences {
    fn from(base: Base) -> Self {
        Self {
            player_name: base.player_name,
            avatar_png: base.avatar_png,
            host_port: base.host_port,
            join_address: base.join_address,
            table_felt_path: base.table_felt_path,
            table_brightness: base.table_brightness,
            table_vignette: base.table_vignette,
            audio_volume: base.audio_volume,
            gender: PlayerGender::Male,
            mahjong_action_voices: true,
            mahjong_fan_voices: true,
            texas_action_voices: true,
        }
    }
}

#[derive(Deserialize, Serialize)]
pub(super) struct Gender<G> {
    pub base: G,
    pub gender: PlayerGender,
}

impl<G: Into<GlobalPreferences>> From<Gender<G>> for GlobalPreferences {
    fn from(value: Gender<G>) -> Self {
        Self {
            gender: value.gender,
            ..value.base.into()
        }
    }
}

#[derive(Deserialize, Serialize)]
pub(super) struct MahjongVoices<G> {
    pub base: G,
    pub action: bool,
    pub fans: bool,
}

impl<G: Into<GlobalPreferences>> From<MahjongVoices<G>> for GlobalPreferences {
    fn from(value: MahjongVoices<G>) -> Self {
        Self {
            mahjong_action_voices: value.action,
            mahjong_fan_voices: value.fans,
            ..value.base.into()
        }
    }
}

#[derive(Deserialize, Serialize)]
pub(super) struct TexasVoice<G> {
    pub base: G,
    pub enabled: bool,
}

impl<G: Into<GlobalPreferences>> From<TexasVoice<G>> for GlobalPreferences {
    fn from(value: TexasVoice<G>) -> Self {
        Self {
            texas_action_voices: value.enabled,
            ..value.base.into()
        }
    }
}

pub(super) type AllVoices = TexasVoice<MahjongVoices<Gender<Base>>>;
