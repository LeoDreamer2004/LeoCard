//! 持久化玩家偏好与 Bevy 连接表单之间的适配。

pub use leocard_client::{
    GamePreferences, GlobalPreferences, MahjongPreferences, QiGui523Preferences, SavedPreferences,
    ShengjiPreferences, TexasHoldemPreferences, UnoPreferences, load_player_preferences,
    save_player_preferences,
};
use leocard_mahjong::MahjongRuleSet;
use leocard_protocol::{MAX_PLAYER_NAME_CHARS, TABLE_SEAT_COUNT};
use leocard_qigui523::QiGuiRuleSet;
use leocard_shengji::ShengjiRuleSet;
use leocard_texas_holdem::TexasHoldemRuleSet;
use leocard_uno::UnoRuleSet;
use std::path::PathBuf;

use super::*;

const DEFAULT_AUDIO_VOLUME: f32 = 0.8;

#[derive(Resource)]
pub struct ConnectionForm {
    pub player_name: String,
    pub avatar_png: Option<Vec<u8>>,
    pub host_port: String,
    pub join_address: String,
    pub table_felt_path: Option<PathBuf>,
    pub table_brightness: f32,
    pub table_vignette: f32,
    pub audio_volume: f32,
    pub host_rules: QiGuiRuleSet,
    pub texas_holdem_rules: TexasHoldemRuleSet,
    pub shengji_rules: ShengjiRuleSet,
    pub uno_rules: UnoRuleSet,
    pub mahjong_rules: MahjongRuleSet,
    pub active: InputField,
    pub error: Option<String>,
}

pub fn truncate_chars(value: &str, maximum: usize) -> String {
    value.chars().take(maximum).collect()
}

pub fn normalize_table_brightness(value: f32) -> f32 {
    if value.is_finite() && (MIN_TABLE_BRIGHTNESS..=MAX_TABLE_BRIGHTNESS).contains(&value) {
        value
    } else {
        1.0
    }
}

pub fn normalize_range(value: f32, minimum: f32, maximum: f32, fallback: f32) -> f32 {
    if value.is_finite() && (minimum..=maximum).contains(&value) {
        value
    } else {
        fallback
    }
}

impl Default for ConnectionForm {
    fn default() -> Self {
        let saved = load_player_preferences().unwrap_or_default();
        let saved_player_name = saved.global.player_name.trim();
        Self {
            player_name: if saved_player_name.is_empty() {
                "玩家".to_owned()
            } else {
                truncate_chars(saved_player_name, MAX_PLAYER_NAME_CHARS)
            },
            // Validate and normalize the persisted avatar before loading it into Bevy.
            avatar_png: saved
                .global
                .avatar_png
                .and_then(|png| normalize_avatar_bytes(&png).ok()),
            host_port: if saved
                .global
                .host_port
                .parse::<u16>()
                .is_ok_and(|port| port > 0)
            {
                saved.global.host_port
            } else {
                "52300".to_owned()
            },
            join_address: if saved.global.join_address.trim().is_empty() {
                "127.0.0.1:52300".to_owned()
            } else {
                saved.global.join_address
            },
            table_felt_path: saved.global.table_felt_path,
            table_brightness: normalize_table_brightness(saved.global.table_brightness),
            table_vignette: normalize_range(
                saved.global.table_vignette,
                MIN_TABLE_VIGNETTE,
                MAX_TABLE_VIGNETTE,
                DEFAULT_TABLE_VIGNETTE,
            ),
            audio_volume: normalize_range(
                saved.global.audio_volume,
                0.0,
                1.0,
                DEFAULT_AUDIO_VOLUME,
            ),
            host_rules: normalize_host_rules(saved.games.qigui523.host_rules),
            texas_holdem_rules: normalize_texas_holdem_rules(saved.games.texas_holdem.host_rules),
            shengji_rules: normalize_shengji_rules(saved.games.shengji.host_rules),
            uno_rules: normalize_uno_rules(saved.games.uno.host_rules),
            mahjong_rules: normalize_mahjong_rules(saved.games.mahjong.host_rules),
            active: InputField::PlayerName,
            error: None,
        }
    }
}

pub fn normalize_host_rules(rules: QiGuiRuleSet) -> QiGuiRuleSet {
    let rules = QiGuiRuleSet {
        player_count: TABLE_SEAT_COUNT,
        developer_deck: if cfg!(feature = "developer") {
            rules.developer_deck
        } else {
            false
        },
        ..rules
    };
    rules.validate().unwrap_or_else(|_| QiGuiRuleSet {
        player_count: TABLE_SEAT_COUNT,
        ..QiGuiRuleSet::default()
    })
}

pub fn normalize_texas_holdem_rules(rules: TexasHoldemRuleSet) -> TexasHoldemRuleSet {
    let rules = TexasHoldemRuleSet {
        player_count: TexasHoldemRuleSet::MAX_PLAYERS,
        ..rules
    };
    rules.validate().unwrap_or_else(|_| TexasHoldemRuleSet {
        player_count: TexasHoldemRuleSet::MAX_PLAYERS,
        ..TexasHoldemRuleSet::default()
    })
}

pub fn normalize_shengji_rules(rules: ShengjiRuleSet) -> ShengjiRuleSet {
    rules.validate().unwrap_or_default()
}

pub fn normalize_uno_rules(rules: UnoRuleSet) -> UnoRuleSet {
    rules.validate().unwrap_or_default()
}

pub fn normalize_mahjong_rules(rules: MahjongRuleSet) -> MahjongRuleSet {
    rules.validate().unwrap_or_default()
}

pub fn save_preferences(form: &ConnectionForm) -> Result<(), String> {
    save_player_preferences(&SavedPreferences {
        global: GlobalPreferences {
            player_name: form.player_name.clone(),
            avatar_png: form.avatar_png.clone(),
            host_port: form.host_port.clone(),
            join_address: form.join_address.clone(),
            table_felt_path: form.table_felt_path.clone(),
            table_brightness: form.table_brightness,
            table_vignette: form.table_vignette,
            audio_volume: form.audio_volume,
        },
        games: GamePreferences {
            qigui523: QiGui523Preferences {
                host_rules: form.host_rules,
            },
            texas_holdem: TexasHoldemPreferences {
                host_rules: form.texas_holdem_rules,
            },
            shengji: ShengjiPreferences {
                host_rules: form.shengji_rules,
            },
            uno: UnoPreferences {
                host_rules: form.uno_rules,
            },
            mahjong: MahjongPreferences {
                host_rules: form.mahjong_rules,
            },
        },
    })
}
