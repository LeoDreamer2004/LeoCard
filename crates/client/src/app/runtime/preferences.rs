//! 持久化玩家偏好与 Bevy 连接表单之间的适配。

use super::normalize_avatar_bytes;
use crate::app::presentation::{
    DEFAULT_TABLE_VIGNETTE, MAX_TABLE_BRIGHTNESS, MAX_TABLE_VIGNETTE, MIN_TABLE_BRIGHTNESS,
    MIN_TABLE_VIGNETTE,
};
use crate::app::shell::InputField;
use bevy::prelude::*;
pub(crate) use leocard_client::{
    GamePreferences, MahjongPreferences, QiGui523Preferences, SavedPreferences, ShengjiPreferences,
    TexasHoldemPreferences, UnoPreferences, load_player_preferences, save_player_preferences,
};
use leocard_mahjong::MahjongRuleSet;
use leocard_protocol::{GameKind, GameRules, MAX_PLAYER_NAME_CHARS, TABLE_SEAT_COUNT};
use leocard_qigui523::QiGuiRuleSet;
use leocard_shengji::ShengjiRuleSet;
use leocard_texas_holdem::TexasHoldemRuleSet;
use leocard_uno::UnoRuleSet;
use std::path::PathBuf;

const DEFAULT_AUDIO_VOLUME: f32 = 0.8;

#[derive(Resource)]
pub(crate) struct ConnectionDraft {
    pub player_name: String,
    pub host_port: String,
    pub join_address: String,
    pub active: InputField,
}

#[derive(Resource)]
pub(crate) struct AppearancePreferences {
    pub avatar_png: Option<Vec<u8>>,
    pub table_felt_path: Option<PathBuf>,
    pub table_brightness: f32,
    pub table_vignette: f32,
    pub audio_volume: f32,
}

#[derive(Resource)]
pub(crate) struct HostRulePreferences {
    pub host_rules: QiGuiRuleSet,
    pub texas_holdem_rules: TexasHoldemRuleSet,
    pub shengji_rules: ShengjiRuleSet,
    pub uno_rules: UnoRuleSet,
    pub mahjong_rules: MahjongRuleSet,
}

#[derive(Resource, Default)]
pub(crate) struct PageErrorState {
    pub error: Option<String>,
}

pub(super) struct PreferenceResources {
    pub connection: ConnectionDraft,
    pub appearance: AppearancePreferences,
    pub host_rules: HostRulePreferences,
}

pub(crate) fn truncate_chars(value: &str, maximum: usize) -> String {
    value.chars().take(maximum).collect()
}

pub(crate) fn normalize_table_brightness(value: f32) -> f32 {
    if value.is_finite() && (MIN_TABLE_BRIGHTNESS..=MAX_TABLE_BRIGHTNESS).contains(&value) {
        value
    } else {
        1.0
    }
}

pub(crate) fn normalize_range(value: f32, minimum: f32, maximum: f32, fallback: f32) -> f32 {
    if value.is_finite() && (minimum..=maximum).contains(&value) {
        value
    } else {
        fallback
    }
}

impl PreferenceResources {
    pub(super) fn load() -> Self {
        Self::from_saved(load_player_preferences().unwrap_or_default())
    }

    fn from_saved(saved: SavedPreferences) -> Self {
        let saved_player_name = saved.global.player_name.trim();
        Self {
            connection: ConnectionDraft {
                player_name: if saved_player_name.is_empty() {
                    "玩家".to_owned()
                } else {
                    truncate_chars(saved_player_name, MAX_PLAYER_NAME_CHARS)
                },
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
                active: InputField::PlayerName,
            },
            appearance: AppearancePreferences {
                avatar_png: saved
                    .global
                    .avatar_png
                    .and_then(|png| normalize_avatar_bytes(&png).ok()),
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
            },
            host_rules: HostRulePreferences {
                host_rules: normalize_rules(saved.games.qigui523.host_rules),
                texas_holdem_rules: normalize_rules(saved.games.texas_holdem.host_rules),
                shengji_rules: normalize_rules(saved.games.shengji.host_rules),
                uno_rules: normalize_rules(saved.games.uno.host_rules),
                mahjong_rules: normalize_rules(saved.games.mahjong.host_rules),
            },
        }
    }
}

impl Default for ConnectionDraft {
    fn default() -> Self {
        PreferenceResources::load().connection
    }
}

impl Default for AppearancePreferences {
    fn default() -> Self {
        PreferenceResources::load().appearance
    }
}

impl Default for HostRulePreferences {
    fn default() -> Self {
        PreferenceResources::load().host_rules
    }
}

pub(crate) trait NormalizeRuleSet: Copy {
    fn normalized(self) -> Self;
}

pub(crate) fn normalize_rules<R: NormalizeRuleSet>(rules: R) -> R {
    rules.normalized()
}

impl NormalizeRuleSet for QiGuiRuleSet {
    fn normalized(self) -> Self {
        let rules = Self {
            player_count: TABLE_SEAT_COUNT,
            developer_deck: cfg!(feature = "developer") && self.developer_deck,
            ..self
        };
        rules.validate().unwrap_or_else(|_| Self {
            player_count: TABLE_SEAT_COUNT,
            ..Self::default()
        })
    }
}

impl NormalizeRuleSet for TexasHoldemRuleSet {
    fn normalized(self) -> Self {
        let rules = Self {
            player_count: Self::MAX_PLAYERS,
            ..self
        };
        rules.validate().unwrap_or_else(|_| Self {
            player_count: Self::MAX_PLAYERS,
            ..Self::default()
        })
    }
}

macro_rules! impl_validated_rule_set {
    ($($rules:ty),+ $(,)?) => {
        $(
            impl NormalizeRuleSet for $rules {
                fn normalized(self) -> Self {
                    self.validate().unwrap_or_default()
                }
            }
        )+
    };
}

impl_validated_rule_set!(ShengjiRuleSet, UnoRuleSet, MahjongRuleSet);

impl HostRulePreferences {
    pub(crate) fn game_rules(&self, kind: GameKind) -> GameRules {
        match kind {
            GameKind::QiGui523 => normalize_rules(self.host_rules).into(),
            GameKind::TexasHoldem => normalize_rules(self.texas_holdem_rules).into(),
            GameKind::Shengji => normalize_rules(self.shengji_rules).into(),
            GameKind::Uno => normalize_rules(self.uno_rules).into(),
            GameKind::Mahjong => normalize_rules(self.mahjong_rules).into(),
        }
    }

    pub(super) fn accept_game_rules(&mut self, rules: &GameRules) -> bool {
        match rules {
            GameRules::QiGui523(rules) => {
                replace_if_changed(&mut self.host_rules, normalize_rules(*rules))
            }
            GameRules::TexasHoldem(rules) => {
                replace_if_changed(&mut self.texas_holdem_rules, normalize_rules(*rules))
            }
            GameRules::Shengji(rules) => {
                replace_if_changed(&mut self.shengji_rules, normalize_rules(*rules))
            }
            GameRules::Uno(rules) => {
                replace_if_changed(&mut self.uno_rules, normalize_rules(*rules))
            }
            GameRules::Mahjong(rules) => {
                replace_if_changed(&mut self.mahjong_rules, normalize_rules(*rules))
            }
        }
    }
}

fn replace_if_changed<T: PartialEq>(slot: &mut T, value: T) -> bool {
    if *slot == value {
        return false;
    }
    *slot = value;
    true
}

pub(crate) fn save_connection_draft(draft: &ConnectionDraft) -> Result<(), String> {
    let mut saved = load_player_preferences().unwrap_or_default();
    saved.global.player_name.clone_from(&draft.player_name);
    saved.global.host_port.clone_from(&draft.host_port);
    saved.global.join_address.clone_from(&draft.join_address);
    save_player_preferences(&saved)
}

pub(crate) fn save_appearance_preferences(
    appearance: &AppearancePreferences,
) -> Result<(), String> {
    let mut saved = load_player_preferences().unwrap_or_default();
    saved.global.avatar_png.clone_from(&appearance.avatar_png);
    saved
        .global
        .table_felt_path
        .clone_from(&appearance.table_felt_path);
    saved.global.table_brightness = appearance.table_brightness;
    saved.global.table_vignette = appearance.table_vignette;
    saved.global.audio_volume = appearance.audio_volume;
    save_player_preferences(&saved)
}

pub(super) fn save_host_rule_preferences(rules: &HostRulePreferences) -> Result<(), String> {
    let mut saved = load_player_preferences().unwrap_or_default();
    saved.games = GamePreferences {
        qigui523: QiGui523Preferences {
            host_rules: rules.host_rules,
        },
        texas_holdem: TexasHoldemPreferences {
            host_rules: rules.texas_holdem_rules,
        },
        shengji: ShengjiPreferences {
            host_rules: rules.shengji_rules,
        },
        uno: UnoPreferences {
            host_rules: rules.uno_rules,
        },
        mahjong: MahjongPreferences {
            host_rules: rules.mahjong_rules,
        },
    };
    save_player_preferences(&saved)
}
