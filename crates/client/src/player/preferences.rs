use super::config_file;
use leocard_mahjong::MahjongRuleSet;
use leocard_protocol::TABLE_SEAT_COUNT;
use leocard_qigui523::QiGuiRuleSet;
use leocard_shengji::ShengjiRuleSet;
use leocard_texas_holdem::TexasHoldemRuleSet;
use leocard_uno::UnoRuleSet;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const DEFAULT_AUDIO_VOLUME: f32 = 0.8;
const DEFAULT_TABLE_VIGNETTE: f32 = 0.38;

#[derive(Deserialize, Serialize)]
pub struct SavedPreferences {
    pub global: GlobalPreferences,
    pub games: GamePreferences,
}

#[derive(Deserialize, Serialize)]
pub struct GlobalPreferences {
    pub player_name: String,
    pub avatar_png: Option<Vec<u8>>,
    pub host_port: String,
    pub join_address: String,
    pub table_felt_path: Option<PathBuf>,
    pub table_brightness: f32,
    pub table_vignette: f32,
    #[serde(default = "default_audio_volume")]
    pub audio_volume: f32,
}

#[derive(Deserialize, Serialize)]
pub struct GamePreferences {
    pub qigui523: QiGui523Preferences,
    pub texas_holdem: TexasHoldemPreferences,
    pub shengji: ShengjiPreferences,
    pub uno: UnoPreferences,
    pub mahjong: MahjongPreferences,
}

#[derive(Deserialize, Serialize)]
pub struct QiGui523Preferences {
    pub host_rules: QiGuiRuleSet,
}

#[derive(Deserialize, Serialize)]
pub struct TexasHoldemPreferences {
    pub host_rules: TexasHoldemRuleSet,
}

#[derive(Default, Deserialize, Serialize)]
pub struct ShengjiPreferences {
    pub host_rules: ShengjiRuleSet,
}

#[derive(Default, Deserialize, Serialize)]
pub struct UnoPreferences {
    pub host_rules: UnoRuleSet,
}

#[derive(Default, Deserialize, Serialize)]
pub struct MahjongPreferences {
    pub host_rules: MahjongRuleSet,
}

impl Default for SavedPreferences {
    fn default() -> Self {
        Self {
            global: GlobalPreferences {
                player_name: String::new(),
                avatar_png: None,
                host_port: String::new(),
                join_address: String::new(),
                table_felt_path: None,
                table_brightness: 1.0,
                table_vignette: DEFAULT_TABLE_VIGNETTE,
                audio_volume: DEFAULT_AUDIO_VOLUME,
            },
            games: GamePreferences {
                qigui523: QiGui523Preferences {
                    host_rules: QiGuiRuleSet {
                        player_count: TABLE_SEAT_COUNT,
                        ..QiGuiRuleSet::default()
                    },
                },
                texas_holdem: TexasHoldemPreferences {
                    host_rules: TexasHoldemRuleSet {
                        player_count: TexasHoldemRuleSet::MAX_PLAYERS,
                        ..TexasHoldemRuleSet::default()
                    },
                },
                shengji: ShengjiPreferences::default(),
                uno: UnoPreferences::default(),
                mahjong: MahjongPreferences::default(),
            },
        }
    }
}

fn default_audio_volume() -> f32 {
    DEFAULT_AUDIO_VOLUME
}

pub fn load_player_preferences() -> Option<SavedPreferences> {
    let bytes = fs::read(config_file("client.prefs")?).ok()?;
    postcard::from_bytes(&bytes).ok()
}

pub fn save_player_preferences(preferences: &SavedPreferences) -> Result<(), String> {
    let path = config_file("client.prefs").ok_or_else(|| "无法确定本机配置目录".to_owned())?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| format!("无法创建配置目录：{error}"))?;
    }
    let bytes =
        postcard::to_allocvec(preferences).map_err(|error| format!("配置编码失败：{error}"))?;
    fs::write(path, bytes).map_err(|error| format!("无法保存本地配置：{error}"))
}
