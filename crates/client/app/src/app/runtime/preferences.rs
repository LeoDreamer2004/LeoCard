//! 持久化玩家偏好与 Bevy 连接表单之间的适配。

use super::{TableSurfaceKind, normalize_avatar_bytes};
use crate::app::presentation::{MAX_TABLE_BRIGHTNESS, MIN_TABLE_BRIGHTNESS};
use bevy::prelude::*;
use leocard_blackjack::BlackjackRuleSet;
use leocard_client::PlayerEconomy;
pub(crate) use leocard_client::{
    DoudizhuPreferences, GameAppearancePreferences, GamePreferences, GlobalPreferences,
    GuandanPreferences, MahjongPreferences, McGuessPreferences, QiGui523Preferences,
    SavedPreferences, ShengjiPreferences, TexasPreferences, UnoPreferences,
    load_player_preferences, save_game_preferences, save_global_preferences,
};
use leocard_doudizhu::DoudizhuRuleSet;
use leocard_guandan::GuandanRuleSet;
use leocard_mahjong::MahjongRuleSet;
use leocard_mc_guess::McGuessRuleSet;
use leocard_protocol::{
    GameKind, GameRules, MAX_PLAYER_NAME_CHARS, PlayerGender, TABLE_SEAT_COUNT,
};
use leocard_qigui523::QiGuiRuleSet;
use leocard_shengji::ShengjiRuleSet;
use leocard_texas_holdem::TexasRuleSet;
use leocard_uno::UnoRuleSet;
use std::path::PathBuf;

const DEFAULT_AUDIO_VOLUME: f32 = 0.8;

#[derive(Resource)]
pub(crate) struct ConnectionDraft {
    pub player_name: String,
    pub gender: PlayerGender,
    pub host_port: String,
    pub join_address: String,
}

#[derive(Resource)]
pub(crate) struct AppearancePreferences {
    pub games: GameAppearancePreferences,
    pub avatar_png: Option<Vec<u8>>,
    pub table_felt_path: Option<PathBuf>,
    pub table_brightness: f32,
    pub mahjong_table_felt_path: Option<PathBuf>,
    pub mahjong_table_brightness: f32,
    pub audio_volume: f32,
    pub turn_notifications: bool,
    pub mahjong_action_voices: bool,
    pub mahjong_fan_voices: bool,
    pub mahjong_tile_voices: bool,
    pub texas_action_voices: bool,
    pub qigui_action_voices: bool,
    pub doudizhu_action_voices: bool,
    pub guandan_action_voices: bool,
}

impl AppearancePreferences {
    pub(crate) fn felt_path(&self, kind: TableSurfaceKind) -> &Option<PathBuf> {
        match kind {
            TableSurfaceKind::Cards => &self.table_felt_path,
            TableSurfaceKind::Mahjong => &self.mahjong_table_felt_path,
        }
    }

    pub(crate) fn felt_path_mut(&mut self, kind: TableSurfaceKind) -> &mut Option<PathBuf> {
        match kind {
            TableSurfaceKind::Cards => &mut self.table_felt_path,
            TableSurfaceKind::Mahjong => &mut self.mahjong_table_felt_path,
        }
    }

    pub(crate) fn brightness(&self, kind: TableSurfaceKind) -> f32 {
        match kind {
            TableSurfaceKind::Cards => self.table_brightness,
            TableSurfaceKind::Mahjong => self.mahjong_table_brightness,
        }
    }

    pub(crate) fn set_brightness(&mut self, kind: TableSurfaceKind, brightness: f32) {
        match kind {
            TableSurfaceKind::Cards => self.table_brightness = brightness,
            TableSurfaceKind::Mahjong => self.mahjong_table_brightness = brightness,
        }
    }
}

#[derive(Resource)]
pub(crate) struct HostRulePreferences {
    pub host_rules: QiGuiRuleSet,
    pub texas_rules: TexasRuleSet,
    pub shengji_rules: ShengjiRuleSet,
    pub shengji_modes: [ShengjiRuleSet; 2],
    pub uno_rules: UnoRuleSet,
    pub mahjong_rules: MahjongRuleSet,
    pub mc_guess_rules: McGuessRuleSet,
    pub doudizhu_rules: DoudizhuRuleSet,
    pub doudizhu_modes: [DoudizhuRuleSet; 2],
    pub guandan_rules: GuandanRuleSet,
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
        GlobalPreferences::default().table_brightness
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
        Self::from_saved(load_player_preferences().unwrap_or_else(|error| {
            warn!("{error}");
            SavedPreferences::default()
        }))
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
                gender: saved.global.gender,
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
            },
            appearance: AppearancePreferences {
                games: GameAppearancePreferences::load().unwrap_or_else(|error| {
                    warn!("{error}");
                    GameAppearancePreferences::default()
                }),
                avatar_png: saved
                    .global
                    .avatar_png
                    .and_then(|png| normalize_avatar_bytes(&png).ok()),
                table_felt_path: saved.global.table_felt_path,
                table_brightness: normalize_table_brightness(saved.global.table_brightness),
                mahjong_table_felt_path: saved.global.mahjong_table_felt_path,
                mahjong_table_brightness: normalize_table_brightness(
                    saved.global.mahjong_table_brightness,
                ),
                audio_volume: normalize_range(
                    saved.global.audio_volume,
                    0.0,
                    1.0,
                    DEFAULT_AUDIO_VOLUME,
                ),
                turn_notifications: saved.global.turn_notifications,
                mahjong_action_voices: saved.global.mahjong_action_voices,
                mahjong_fan_voices: saved.global.mahjong_fan_voices,
                mahjong_tile_voices: saved.global.mahjong_tile_voices,
                texas_action_voices: saved.global.texas_action_voices,
                qigui_action_voices: saved.global.qigui_action_voices,
                doudizhu_action_voices: saved.global.doudizhu_action_voices,
                guandan_action_voices: saved.global.guandan_action_voices,
            },
            host_rules: HostRulePreferences {
                host_rules: normalize_rules(saved.games.qigui523.host_rules),
                texas_rules: normalize_rules(saved.games.texas.host_rules),
                shengji_rules: normalize_rules(saved.games.shengji.host_rules),
                shengji_modes: saved.games.shengji.remembered_modes().map(normalize_rules),
                uno_rules: normalize_rules(saved.games.uno.host_rules),
                mahjong_rules: normalize_rules(saved.games.mahjong.host_rules),
                mc_guess_rules: normalize_rules(saved.games.mc_guess.host_rules),
                doudizhu_rules: normalize_rules(saved.games.doudizhu.host_rules),
                doudizhu_modes: saved.games.doudizhu.remembered_modes().map(normalize_rules),
                guandan_rules: normalize_rules(saved.games.guandan.host_rules),
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

impl NormalizeRuleSet for TexasRuleSet {
    fn normalized(self) -> Self {
        let rules = Self {
            player_count: Self::MAX_PLAYERS,
            run_it_twice: self.run_it_twice && !self.side_pots,
            ..self
        };
        rules.validate_options().unwrap_or_else(|_| Self {
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

impl NormalizeRuleSet for McGuessRuleSet {
    fn normalized(self) -> Self {
        if self.is_valid() {
            self
        } else {
            Self::default()
        }
    }
}

impl NormalizeRuleSet for DoudizhuRuleSet {
    fn normalized(self) -> Self {
        Self {
            rounds: self.rounds.clamp(1, 5),
            ..self
        }
    }
}

impl NormalizeRuleSet for GuandanRuleSet {
    fn normalized(self) -> Self {
        if self.is_valid() {
            self
        } else {
            Self::default()
        }
    }
}

impl HostRulePreferences {
    pub(crate) fn game_rules(&self, kind: GameKind) -> GameRules {
        match kind {
            GameKind::Doudizhu => normalize_rules(self.doudizhu_rules).into(),
            GameKind::Guandan => normalize_rules(self.guandan_rules).into(),
            GameKind::MahjongLink => leocard_mahjong_link::MahjongLinkRuleSet::default().into(),
            GameKind::Blackjack => BlackjackRuleSet::default().into(),
            GameKind::McGuess => normalize_rules(self.mc_guess_rules).into(),
            GameKind::QiGui523 => normalize_rules(self.host_rules).into(),
            GameKind::Texas => normalize_rules(self.texas_rules).into(),
            GameKind::Shengji => normalize_rules(self.shengji_rules).into(),
            GameKind::Uno => normalize_rules(self.uno_rules).into(),
            GameKind::Mahjong => normalize_rules(self.mahjong_rules).into(),
        }
    }

    pub(super) fn accept_game_rules(&mut self, rules: &GameRules) -> bool {
        match rules {
            GameRules::Blackjack(_) | GameRules::MahjongLink(_) => false,
            GameRules::Doudizhu(rules) => {
                let rules = normalize_rules(*rules);
                replace_if_changed(&mut self.doudizhu_rules, rules)
                    | replace_if_changed(&mut self.doudizhu_modes[rules.mode as usize], rules)
            }
            GameRules::Guandan(rules) => {
                replace_if_changed(&mut self.guandan_rules, normalize_rules(*rules))
            }
            GameRules::McGuess(rules) => {
                replace_if_changed(&mut self.mc_guess_rules, normalize_rules(*rules))
            }
            GameRules::QiGui523(rules) => {
                replace_if_changed(&mut self.host_rules, normalize_rules(*rules))
            }
            GameRules::Texas(rules) => {
                replace_if_changed(&mut self.texas_rules, normalize_rules(*rules))
            }
            GameRules::Shengji(rules) => {
                let rules = normalize_rules(*rules);
                replace_if_changed(&mut self.shengji_rules, rules)
                    | replace_if_changed(&mut self.shengji_modes[rules.mode as usize], rules)
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

pub(crate) fn save_connection_draft(
    draft: &ConnectionDraft,
    economy: &mut PlayerEconomy,
) -> Result<(), String> {
    let mut saved = load_player_preferences()?;
    saved.global.player_name.clone_from(&draft.player_name);
    saved.global.gender = draft.gender;
    saved.global.host_port.clone_from(&draft.host_port);
    saved.global.join_address.clone_from(&draft.join_address);
    save_global_preferences(&saved.global)?;
    economy.sync_exclusive_frame(&draft.player_name)
}

pub(crate) fn save_appearance_preferences(
    appearance: &AppearancePreferences,
) -> Result<(), String> {
    let mut saved = load_player_preferences()?;
    saved.global.avatar_png.clone_from(&appearance.avatar_png);
    saved
        .global
        .table_felt_path
        .clone_from(&appearance.table_felt_path);
    saved.global.table_brightness = appearance.table_brightness;
    saved
        .global
        .mahjong_table_felt_path
        .clone_from(&appearance.mahjong_table_felt_path);
    saved.global.mahjong_table_brightness = appearance.mahjong_table_brightness;
    saved.global.audio_volume = appearance.audio_volume;
    saved.global.turn_notifications = appearance.turn_notifications;
    saved.global.mahjong_action_voices = appearance.mahjong_action_voices;
    saved.global.mahjong_fan_voices = appearance.mahjong_fan_voices;
    saved.global.mahjong_tile_voices = appearance.mahjong_tile_voices;
    saved.global.texas_action_voices = appearance.texas_action_voices;
    saved.global.qigui_action_voices = appearance.qigui_action_voices;
    saved.global.doudizhu_action_voices = appearance.doudizhu_action_voices;
    saved.global.guandan_action_voices = appearance.guandan_action_voices;
    save_global_preferences(&saved.global)
}

pub(super) fn save_host_rule_preferences(rules: &HostRulePreferences) -> Result<(), String> {
    let games = GamePreferences {
        qigui523: QiGui523Preferences {
            host_rules: rules.host_rules,
        },
        texas: TexasPreferences {
            host_rules: rules.texas_rules,
        },
        shengji: ShengjiPreferences {
            host_rules: rules.shengji_rules,
            mode_rules: rules.shengji_modes,
        },
        uno: UnoPreferences {
            host_rules: rules.uno_rules,
        },
        mahjong: MahjongPreferences {
            host_rules: rules.mahjong_rules,
        },
        mc_guess: McGuessPreferences {
            host_rules: normalize_rules(rules.mc_guess_rules),
        },
        doudizhu: DoudizhuPreferences {
            host_rules: normalize_rules(rules.doudizhu_rules),
            mode_rules: rules.doudizhu_modes,
        },
        guandan: GuandanPreferences {
            host_rules: normalize_rules(rules.guandan_rules),
        },
    };
    save_game_preferences(&games)
}
