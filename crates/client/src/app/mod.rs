//! Bevy client application. Feature areas live in focused sibling modules.

use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;

use bevy::asset::{AssetPlugin, RenderAssetUsages};
use bevy::audio::{GlobalVolume, Volume};
use bevy::ecs::system::SystemParam;
use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::log::{DEFAULT_FILTER, LogPlugin};
use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;
use bevy::ui::{FocusPolicy, RelativeCursorPosition, UiScale};
use bevy::window::{FileDragAndDrop, Ime, PrimaryWindow, WindowResizeConstraints};
use bevy_clipboard::Clipboard;
use leocard_client::{
    NetworkState, PlayerIdentity, ScoreCaptureEffect, ShengjiScoreCaptureEffect, TcpGameClient,
    selected_cards_in_hand,
};
use leocard_protocol::{
    AVATAR_DIMENSION, AvatarId, ChatContent, ChatEmoji, ClientCommand, GameCommand, GameKind,
    GamePhaseView, GameViolation, MAX_AVATAR_BYTES, MAX_CHAT_MESSAGE_CHARS, MAX_PLAYER_NAME_CHARS,
    MatchId, PlayerGameProfiles, PlayerId, PlayerInteractionKind, PlayerInteractionStats,
    PlayerPublicState, PlayerScore, PublicPlay, PublicPlayRecord, QUICK_VOICE_COUNT,
    QiGui523Command, QiGui523ProfileStats, RejectReason, RuleViolation, SeatId, ShengjiCommand,
    ShengjiFiveTrumpCrossingStage, ShengjiPhaseView, ShengjiPlayerState, ShengjiProfileStats,
    ShengjiPublicPlay, ShengjiSnapshot, ShengjiThrowFailureStage, ShengjiViolation,
    TABLE_SEAT_COUNT, TexasHoldemCommand, TexasHoldemEvent, TexasHoldemPhaseView,
    TexasHoldemPlayerState, TexasHoldemProfileStats, TexasHoldemSnapshot, TexasHoldemViolation,
    TurnTimerView, UnoCommand, UnoEvent, UnoPendingSwapView, UnoPhaseView, UnoPlayerState,
    UnoProfileStats, UnoSnapshot, UnoViolation,
};
use leocard_qigui523::{
    Card, ClassifiedPlay, PlayKind, QiGui523Bot, QiGui523BotRequest, Rank, RuleSet, SameCardPolicy,
    Suit, SuitComparison, TimeControl, build_deck, classify, has_legal_response,
};
use leocard_shengji::{
    BidTrump as ShengjiBidTrump, Card as ShengjiCard, GreedyBot as ShengjiGreedyBot,
    GreedyBotRequest as ShengjiGreedyBotRequest, Rank as ShengjiRank, RuleSet as ShengjiRuleSet,
    Suit as ShengjiSuit, ThrowPenalty as ShengjiThrowPenalty, Trump as ShengjiTrump,
    bid_joker_for_suit as shengji_bid_joker_for_suit,
    follow_suggestions as shengji_follow_suggestions,
    forced_follow_cards as shengji_forced_follow_cards,
};
use leocard_texas_holdem::{
    Action as TexasHoldemAction, BlindKind as TexasBlindKind, Card as TexasHoldemCard,
    HandCategory as TexasHandCategory, Rank as TexasRank, RuleSet as TexasHoldemRuleSet,
    Street as TexasStreet, Suit as TexasSuit,
};
use leocard_uno::{
    Card as UnoCard, ChallengeResult as UnoChallengeResult, Color as UnoColor,
    Direction as UnoDirection, Face as UnoFace, PendingDrawKind as UnoPendingDrawKind,
    RuleSet as UnoRuleSet, build_deck_for_rules as build_uno_deck_for_rules,
};
use serde::{Deserialize, Serialize};

const DESIGN_WIDTH: f32 = 1280.0;
const DESIGN_HEIGHT: f32 = 720.0;
const MIN_AUTO_SCALE: f32 = 0.5;
const MAX_AUTO_SCALE: f32 = 2.5;
const MIN_MANUAL_ZOOM: f32 = 0.7;
const MAX_MANUAL_ZOOM: f32 = 1.5;
const HAND_CARD_REVEAL: f32 = 28.0;
const HAND_CARD_SELECTED_LIFT: f32 = 24.0;
const HAND_CARD_HOVER_WIDTH: f32 = 32.0;
const TABLE_CARD_REVEAL: f32 = 27.0;
const SCORE_CARD_REVEAL: f32 = 12.0;
const TABLE_SCORE_CARD_REVEAL: f32 = 8.0;
const FINISHED_HAND_CARD_REVEAL: f32 = 14.4;
const SCORE_CAPTURE_TRAVEL_DURATION: f32 = 0.42;
const SCORE_ROLL_DELAY: f32 = 0.42;
const SCORE_ROLL_DURATION: f32 = 0.72;
const UI_FONT_ASSET: &str = "fonts/ChillRoundGothic-Medium.ttf";
const TABLE_FELT_ASSET: &str = "vendor/opengameart/green-textile/table_felt_dark_green.png";
const MIN_TABLE_BRIGHTNESS: f32 = 0.1;
const MAX_TABLE_BRIGHTNESS: f32 = 1.25;
const MIN_TABLE_VIGNETTE: f32 = 0.0;
const MAX_TABLE_VIGNETTE: f32 = 0.75;
const DEFAULT_TABLE_VIGNETTE: f32 = 0.38;
const DEFAULT_AUDIO_VOLUME: f32 = 0.8;
const TABLE_BACKGROUND_SHADER: &str = "shaders/table_background.wgsl";
const PLAY_ERROR_TOAST_DURATION: f32 = 2.4;
const PLAY_ERROR_TOAST_ENTRY_DURATION: f32 = 0.28;
const PLAY_ERROR_TOAST_FADE_DURATION: f32 = 0.42;
const PLAY_ERROR_TOAST_SHAKE_DURATION: f32 = 0.42;
const SUMMARY_MODAL_ENTRY_DURATION: f32 = 0.55;
const SUMMARY_HAND_REVEAL_DURATION: f32 = 1.5;
/// UNO 最后一张牌的飞行动画结束后，完整公开牌桌两秒再进入结算。
const UNO_PLAY_CARD_DURATION: f32 = 0.58;
const UNO_FINISH_REVEAL_DURATION: f32 = UNO_PLAY_CARD_DURATION + 2.0;
const TEXAS_SHOWDOWN_REVEAL_DURATION: f32 = 2.6;
const TEXAS_UNCONTESTED_REVEAL_DURATION: f32 = 0.8;
const SUMMARY_ROW_START_DELAY: f32 = 0.38;
const SUMMARY_ROW_INTERVAL: f32 = 0.18;
const SUMMARY_ROW_ENTRY_DURATION: f32 = 0.32;
const SUMMARY_SCORE_COUNT_DURATION: f32 = 0.72;
const SUMMARY_ACTIONS_EXTRA_DELAY: f32 = 0.30;
const SHENGJI_KITTY_CARD_INTERVAL: f32 = 0.075;
const SHENGJI_KITTY_CARD_ENTRY_DURATION: f32 = 0.22;
const SHENGJI_KITTY_SCORE_DELAY: f32 = 0.72;
const SHENGJI_KITTY_MULTIPLIER_DELAY: f32 = 1.12;
const SHENGJI_KITTY_MULTIPLIER_DURATION: f32 = 0.62;
const SHENGJI_TOTAL_LABEL_DELAY: f32 = 1.92;
const SHENGJI_TOTAL_ABSORB_DELAY: f32 = 2.24;
const SHENGJI_TOTAL_ABSORB_DURATION: f32 = 0.78;
const SHENGJI_SETTLEMENT_MODAL_DELAY: f32 = 3.38;
const SHENGJI_SETTLEMENT_ROW_INTERVAL: f32 = 0.18;
const INTERACTION_TRAVEL_DURATION: f32 = 0.60;
const INTERACTION_IMPACT_DURATION: f32 = 1.20;
const INTERACTION_APPEAR_DURATION: f32 = 0.32;
const INTERACTION_COOLDOWN_MASK_FRAMES: usize = 48;
const HEAVY_INTERACTION_TRAVEL_DURATION: f32 = 0.68;
const SHOE_ROTATIONS: f32 = 2.0;
const CHAT_PANEL_WIDTH: f32 = 350.0;
/// 将面板本体移出右侧，同时保留其左侧的 32px 折叠箭头。
const CHAT_PANEL_HIDDEN_OFFSET: f32 = CHAT_PANEL_WIDTH + 2.0;
const CHAT_HISTORY_LIMIT: usize = 60;
const START_GAME_SEAT_MOVE_DURATION: f32 = 0.72;

const QUICK_VOICES: [&str; QUICK_VOICE_COUNT as usize] = [
    "我从未见过如此厚颜无耻之人！",
    "这波不亏",
    "请收下我的膝盖",
    "你咋不上天呢",
    "放开我的队友，冲我来",
    "你随便杀，闪不了算我输",
    "见证奇迹的时刻到了",
    "能不能快一点啊，兵贵神速啊",
    "主公，别开枪，自己人",
    "小内再不跳，后面还怎么玩儿啊",
    "你们忍心，就这么让我酱油了？",
    "我，我惹你们了吗",
    "姑娘，你真是条汉子",
    "三十六计，走为上，容我去去便回",
    "人心散了，队伍不好带啊",
    "昏君，昏君啊！",
    "风吹鸡蛋壳，牌去人安乐",
    "小内啊，您老悠着点儿",
    "不好意思，刚才卡了",
    "你可以打得再烂一点吗",
    "哥们，给力点儿行嘛",
    "哥哥，交个朋友吧",
    "妹子，交个朋友吧",
];

const TABLE_BG: Color = Color::srgb(0.025, 0.105, 0.075);
const HEADER_BG: Color = Color::srgb(0.025, 0.075, 0.06);
const PANEL: Color = Color::srgba(0.055, 0.19, 0.135, 0.96);
const PANEL_ALT: Color = Color::srgba(0.075, 0.24, 0.17, 0.96);
const TEXT: Color = Color::srgb(0.94, 0.97, 0.95);
const MUTED: Color = Color::srgb(0.63, 0.73, 0.68);
const ACCENT: Color = Color::srgb(0.96, 0.72, 0.20);
const READY: Color = Color::srgb(0.34, 0.86, 0.53);
const DANGER: Color = Color::srgb(0.96, 0.39, 0.34);
const BORDER: Color = Color::srgba(0.72, 0.88, 0.79, 0.18);
const STRAIGHT_EFFECT: Color = Color::srgb(0.25, 0.78, 0.96);
const CONSECUTIVE_PAIRS_EFFECT: Color = Color::srgb(0.76, 0.46, 0.98);
const AIRPLANE_EFFECT: Color = Color::srgb(0.98, 0.36, 0.50);
const INTERACTION_SOUND_VOLUME: f32 = 0.5;

#[derive(Resource)]
struct ClientResource(TcpGameClient);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum InputField {
    PlayerName,
    HostPort,
    JoinAddress,
}

#[derive(Resource, Default)]
struct DeveloperHandInput {
    value: String,
    focused: bool,
}

#[derive(Resource)]
struct ConnectionForm {
    player_name: String,
    avatar_png: Option<Vec<u8>>,
    host_port: String,
    join_address: String,
    table_felt_path: Option<PathBuf>,
    table_brightness: f32,
    table_vignette: f32,
    audio_volume: f32,
    host_rules: RuleSet,
    texas_holdem_rules: TexasHoldemRuleSet,
    shengji_rules: ShengjiRuleSet,
    uno_rules: UnoRuleSet,
    active: InputField,
    error: Option<String>,
}

fn truncate_chars(value: &str, maximum: usize) -> String {
    value.chars().take(maximum).collect()
}

fn normalize_table_brightness(value: f32) -> f32 {
    if value.is_finite() && (MIN_TABLE_BRIGHTNESS..=MAX_TABLE_BRIGHTNESS).contains(&value) {
        value
    } else {
        1.0
    }
}

fn normalize_range(value: f32, minimum: f32, maximum: f32, fallback: f32) -> f32 {
    if value.is_finite() && (minimum..=maximum).contains(&value) {
        value
    } else {
        fallback
    }
}

impl Default for ConnectionForm {
    fn default() -> Self {
        let saved = load_preferences().unwrap_or_default();
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
            active: InputField::PlayerName,
            error: None,
        }
    }
}

#[derive(Deserialize, Serialize)]
struct SavedPreferences {
    global: GlobalPreferences,
    games: GamePreferences,
}

/// 与所选的棋牌游戏无关、供整个 LeoCard 客户端共享的个人偏好。
#[derive(Deserialize, Serialize)]
struct GlobalPreferences {
    player_name: String,
    avatar_png: Option<Vec<u8>>,
    host_port: String,
    join_address: String,
    table_felt_path: Option<PathBuf>,
    table_brightness: f32,
    table_vignette: f32,
    #[serde(default = "default_audio_volume")]
    audio_volume: f32,
}

fn default_audio_volume() -> f32 {
    DEFAULT_AUDIO_VOLUME
}

#[derive(Deserialize, Serialize)]
struct GamePreferences {
    qigui523: QiGui523Preferences,
    texas_holdem: TexasHoldemPreferences,
    shengji: ShengjiPreferences,
    uno: UnoPreferences,
}

#[derive(Deserialize, Serialize)]
struct QiGui523Preferences {
    host_rules: RuleSet,
}

#[derive(Deserialize, Serialize)]
struct TexasHoldemPreferences {
    host_rules: TexasHoldemRuleSet,
}

/// “奥马哈”加入前的德州扑克规则磁盘格式。
#[derive(Deserialize, Serialize)]
struct PreOmahaTexasHoldemRuleSet {
    player_count: u8,
    starting_chips: u16,
    short_deck: bool,
    ignore_kickers: bool,
}

impl From<PreOmahaTexasHoldemRuleSet> for TexasHoldemRuleSet {
    fn from(value: PreOmahaTexasHoldemRuleSet) -> Self {
        Self {
            player_count: value.player_count,
            starting_chips: value.starting_chips,
            short_deck: value.short_deck,
            ignore_kickers: value.ignore_kickers,
            omaha: false,
        }
    }
}

#[derive(Deserialize, Serialize)]
struct PreOmahaTexasHoldemPreferences {
    host_rules: PreOmahaTexasHoldemRuleSet,
}

#[derive(Deserialize, Serialize)]
struct PreOmahaSavedPreferences {
    global: GlobalPreferences,
    games: PreOmahaGamePreferences,
}

#[derive(Deserialize, Serialize)]
struct PreOmahaGamePreferences {
    qigui523: QiGui523Preferences,
    texas_holdem: PreOmahaTexasHoldemPreferences,
    shengji: ShengjiPreferences,
    uno: UnoPreferences,
}

/// “只比较最大牌型”加入前的德州规则磁盘格式。
#[derive(Deserialize, Serialize)]
struct PreviousTexasHoldemRuleSet {
    player_count: u8,
    starting_chips: u16,
    short_deck: bool,
}

impl From<PreviousTexasHoldemRuleSet> for TexasHoldemRuleSet {
    fn from(value: PreviousTexasHoldemRuleSet) -> Self {
        Self {
            player_count: value.player_count,
            starting_chips: value.starting_chips,
            short_deck: value.short_deck,
            ignore_kickers: false,
            omaha: false,
        }
    }
}

#[derive(Deserialize, Serialize)]
struct PreviousTexasHoldemPreferences {
    host_rules: PreviousTexasHoldemRuleSet,
}

#[derive(Default, Deserialize, Serialize)]
struct ShengjiPreferences {
    host_rules: ShengjiRuleSet,
}

#[derive(Default, Deserialize, Serialize)]
struct UnoPreferences {
    host_rules: UnoRuleSet,
}

/// “只比较最大牌型”加入前、但已经包含升级设置的磁盘格式。
#[derive(Deserialize, Serialize)]
struct PreviousSavedPreferences {
    global: GlobalPreferences,
    games: PreviousGamePreferences,
}

#[derive(Deserialize, Serialize)]
struct PreviousGamePreferences {
    qigui523: QiGui523Preferences,
    texas_holdem: PreviousTexasHoldemPreferences,
    shengji: ShengjiPreferences,
}

/// `shengji` 偏好加入前的磁盘格式，仅用于无损迁移旧版 `client.prefs`。
#[derive(Deserialize, Serialize)]
struct LegacySavedPreferences {
    global: GlobalPreferences,
    games: LegacyGamePreferences,
}

#[derive(Deserialize, Serialize)]
struct LegacyGamePreferences {
    qigui523: QiGui523Preferences,
    texas_holdem: PreviousTexasHoldemPreferences,
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
                    host_rules: normalize_host_rules(RuleSet::default()),
                },
                texas_holdem: TexasHoldemPreferences {
                    host_rules: normalize_texas_holdem_rules(TexasHoldemRuleSet::default()),
                },
                shengji: ShengjiPreferences {
                    host_rules: normalize_shengji_rules(ShengjiRuleSet::default()),
                },
                uno: UnoPreferences {
                    host_rules: normalize_uno_rules(UnoRuleSet::default()),
                },
            },
        }
    }
}

fn normalize_host_rules(rules: RuleSet) -> RuleSet {
    let rules = RuleSet {
        player_count: TABLE_SEAT_COUNT,
        developer_deck: if cfg!(feature = "developer") {
            rules.developer_deck
        } else {
            false
        },
        ..rules
    };
    rules.validate().unwrap_or_else(|_| RuleSet {
        player_count: TABLE_SEAT_COUNT,
        ..RuleSet::default()
    })
}

fn normalize_texas_holdem_rules(rules: TexasHoldemRuleSet) -> TexasHoldemRuleSet {
    let rules = TexasHoldemRuleSet {
        player_count: TexasHoldemRuleSet::MAX_PLAYERS,
        ..rules
    };
    rules.validate().unwrap_or_else(|_| TexasHoldemRuleSet {
        player_count: TexasHoldemRuleSet::MAX_PLAYERS,
        ..TexasHoldemRuleSet::default()
    })
}

fn normalize_shengji_rules(rules: ShengjiRuleSet) -> ShengjiRuleSet {
    rules.validate().unwrap_or_default()
}

fn normalize_uno_rules(rules: UnoRuleSet) -> UnoRuleSet {
    rules.validate().unwrap_or_default()
}

#[derive(Deserialize, Serialize)]
struct StoredPlayerProfile {
    secret_key: [u8; 32],
    games: StoredGameProfiles,
}

#[derive(Deserialize, Serialize)]
struct StoredGameProfiles {
    qigui523: StoredRatingProfile,
    texas_holdem_stats: Option<TexasHoldemProfileStats>,
    shengji_stats: Option<ShengjiProfileStats>,
    uno_stats: Option<UnoProfileStats>,
    interaction_stats: Option<PlayerInteractionStats>,
}

#[derive(Deserialize, Serialize)]
struct StoredRatingProfile {
    reference_points: i32,
    completed_games: u32,
    applied_matches: Vec<MatchId>,
    qigui523_stats: Option<QiGui523ProfileStats>,
}

#[derive(Deserialize, Serialize)]
struct PreInteractionStoredPlayerProfile {
    secret_key: [u8; 32],
    games: PreInteractionStoredGameProfiles,
}

#[derive(Deserialize, Serialize)]
struct PreInteractionStoredGameProfiles {
    qigui523: StoredRatingProfile,
    texas_holdem_stats: Option<TexasHoldemProfileStats>,
    shengji_stats: Option<ShengjiProfileStats>,
    uno_stats: Option<UnoProfileStats>,
}

#[derive(Deserialize, Serialize)]
struct PreUnoStoredPlayerProfile {
    secret_key: [u8; 32],
    games: PreUnoStoredGameProfiles,
}

#[derive(Deserialize, Serialize)]
struct PreUnoStoredGameProfiles {
    qigui523: StoredRatingProfile,
    texas_holdem_stats: Option<TexasHoldemProfileStats>,
    shengji_stats: Option<ShengjiProfileStats>,
}

#[derive(Deserialize, Serialize)]
struct PreShengjiStoredPlayerProfile {
    secret_key: [u8; 32],
    games: PreShengjiStoredGameProfiles,
}

#[derive(Deserialize, Serialize)]
struct PreShengjiStoredGameProfiles {
    qigui523: StoredRatingProfile,
    texas_holdem_stats: Option<TexasHoldemProfileStats>,
}

#[derive(Deserialize, Serialize)]
struct PreTexasStoredPlayerProfile {
    secret_key: [u8; 32],
    games: PreTexasStoredGameProfiles,
}

#[derive(Deserialize, Serialize)]
struct PreTexasStoredGameProfiles {
    qigui523: StoredRatingProfile,
}

#[derive(Deserialize, Serialize)]
struct PreDetailedStoredPlayerProfile {
    secret_key: [u8; 32],
    games: PreDetailedStoredGameProfiles,
}

#[derive(Deserialize, Serialize)]
struct PreDetailedStoredGameProfiles {
    qigui523: PreDetailedStoredRatingProfile,
}

#[derive(Deserialize, Serialize)]
struct PreDetailedStoredRatingProfile {
    reference_points: i32,
    completed_games: u32,
    applied_matches: Vec<MatchId>,
}

#[derive(Resource)]
struct LocalPlayerProfile {
    identity: PlayerIdentity,
    rating: PlayerRatingProfile,
    game_profiles: PlayerGameProfiles,
}

struct PlayerRatingProfile {
    reference_points: i32,
    completed_games: u32,
    applied_matches: HashSet<MatchId>,
    last_change: Option<(MatchId, i16)>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum ProfileGameTab {
    #[default]
    QiGui523,
    TexasHoldem,
    Shengji,
    Uno,
}

impl ProfileGameTab {
    const ALL: [(Self, &'static str); 4] = [
        (Self::QiGui523, "七鬼五二三"),
        (Self::TexasHoldem, "德州扑克"),
        (Self::Shengji, "升级"),
        (Self::Uno, "UNO"),
    ];
}

#[derive(Resource, Default)]
struct UiState {
    selected: HashSet<Card>,
    selected_shengji: HashSet<ShengjiCard>,
    card_animations: HashMap<Card, CardAnimationState>,
    observed_hand: Vec<Card>,
    shengji_card_animations: HashMap<ShengjiCard, CardAnimationState>,
    observed_shengji_hand: Vec<ShengjiCard>,
    selected_uno: HashSet<UnoCard>,
    uno_swap_targets: Vec<PlayerId>,
    uno_card_animations: HashMap<UnoCard, CardAnimationState>,
    greedy_hint: QiGui523Bot,
    interaction_menu_open: Option<PlayerId>,
    uno_mode_menu_open: bool,
    uno_expansion_settings_open: bool,
    settings_open: bool,
    profile_open: bool,
    player_profile: Option<PlayerProfilePage>,
    profile_game_tab: ProfileGameTab,
    host_game_picker_open: bool,
    texas_raise_to: u32,
    texas_observed_match: Option<MatchId>,
    texas_observed_hand_number: u32,
    texas_observed_community_len: usize,
    shengji_observed_match: Option<MatchId>,
    shengji_observed_hand_number: u32,
    shengji_buried_open: bool,
    uno_color_choice: Option<UnoCard>,
    leaving_room: bool,
    dirty: bool,
}

#[derive(Clone)]
struct PlayerProfilePage {
    name: String,
    avatar: Option<Handle<Image>>,
    reference_points: i32,
    completed_games: u32,
    game_profiles: PlayerGameProfiles,
}

#[derive(Component)]
struct SelectedProfileGameTab;

#[derive(Component)]
struct ProfileGameContent;

#[derive(Component)]
struct ProfileGameTabButton;

#[derive(Component)]
struct ProfileGameColumn;

#[derive(Component)]
struct ProfileStat;

#[derive(Resource, Default)]
struct PlayErrorToast {
    seen_rejection_serial: u64,
    seen_notice_serial: u64,
    observed_form_error: Option<String>,
    observed_appearance_error: Option<String>,
    message: Option<String>,
    elapsed: f32,
    shake_elapsed: Option<f32>,
    entering: bool,
    active: bool,
}

#[derive(Resource, Default)]
struct GameSummaryAnimation {
    match_id: Option<MatchId>,
    texas_hand_number: Option<u32>,
    scores: Vec<(PlayerId, u32)>,
    elapsed: f32,
    nonnegative_outcome: bool,
    outcome_sound_played: bool,
}

#[derive(Resource, Default)]
struct TexasRaiseHoldState {
    direction: i8,
    step: u32,
    minimum: u32,
    maximum: u32,
    elapsed: f32,
    next_repeat: f32,
}

#[derive(Resource, Default)]
struct PlayEffectState {
    seen_serial: u64,
    active: Option<ActivePlayEffect>,
}

#[derive(Resource, Default)]
struct ScoreCaptureEffectState {
    seen_serial: u64,
    active: Option<ActiveScoreCapture>,
}

#[derive(Resource, Default)]
struct ShengjiScoreCaptureEffectState {
    seen_serial: u64,
    active: Option<ActiveShengjiScoreCapture>,
}

#[derive(Resource, Default)]
struct ShengjiSettlementAnimation {
    settlement_id: Option<MatchId>,
    elapsed: f32,
    absorption_spawned: bool,
    outcome_sound_played: bool,
}

#[derive(Clone)]
struct ActiveShengjiScoreCapture {
    capture: ShengjiScoreCaptureEffect,
    elapsed: f32,
}

#[derive(Clone)]
struct ActiveScoreCapture {
    capture: ScoreCaptureEffect,
    elapsed: f32,
}

#[derive(Component)]
struct PendingDealSound {
    remaining: f32,
    variant: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CardPlaySoundKind {
    Place,
    Shove,
}

#[derive(Resource, Default)]
struct PlayerInteractionCooldown {
    timers: HashMap<PlayerInteractionKind, InteractionCooldownTimer>,
}

#[derive(Clone, Copy)]
struct InteractionCooldownTimer {
    remaining: f32,
    duration: f32,
}

impl PlayerInteractionCooldown {
    fn is_active(&self, kind: PlayerInteractionKind) -> bool {
        self.timers
            .get(&kind)
            .is_some_and(|timer| timer.remaining > 0.0)
    }

    fn start(&mut self, kind: PlayerInteractionKind, duration: f32) {
        self.timers.insert(
            kind,
            InteractionCooldownTimer {
                remaining: duration,
                duration,
            },
        );
    }

    fn tick(&mut self, delta: f32) {
        for timer in self.timers.values_mut() {
            timer.remaining = (timer.remaining - delta).max(0.0);
        }
        self.timers.retain(|_, timer| timer.remaining > 0.0);
    }

    fn fraction(&self, kind: PlayerInteractionKind) -> f32 {
        self.timers.get(&kind).map_or(0.0, |timer| {
            (timer.remaining / timer.duration).clamp(0.0, 1.0)
        })
    }
}

#[derive(Clone)]
struct ActivePlayEffect {
    player: PlayerId,
    play: PublicPlay,
    elapsed: f32,
    bomb_sound_played: bool,
}

#[derive(Clone, Copy, Default, PartialEq)]
struct CardAnimationState {
    slot_hover_amount: f32,
    face_hover_amount: f32,
    selected_amount: f32,
    deal_elapsed: f32,
    dealing: bool,
}

#[derive(Resource)]
struct UiZoom {
    manual: f32,
}

#[derive(Resource, Default)]
struct CardDragSelection {
    active: bool,
    anchor: usize,
    current: usize,
    select: bool,
}

#[derive(Resource, Default)]
struct ShengjiCardDragSelection {
    active: bool,
    anchor: usize,
    current: usize,
    select: bool,
}

#[derive(Clone, Debug)]
struct ChatHistoryEntry {
    player_name: String,
    message: String,
}

#[derive(Resource)]
struct ChatPanelState {
    open: bool,
    /// 0 表示完全展开，1 表示完全收起。
    slide: f32,
    focused: bool,
    quick_voice_open: bool,
    emoji_open: bool,
    /// 快捷语音列表的持久滚动位置；UI 根节点重建后据此恢复。
    quick_voice_scroll_y: f32,
    emoji_scroll_y: f32,
    input: String,
    history: VecDeque<ChatHistoryEntry>,
}

impl Default for ChatPanelState {
    fn default() -> Self {
        Self {
            open: false,
            slide: 1.0,
            focused: false,
            quick_voice_open: false,
            emoji_open: false,
            quick_voice_scroll_y: 0.0,
            emoji_scroll_y: 0.0,
            input: String::new(),
            history: VecDeque::new(),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
enum HintDecision {
    Select(Vec<Card>),
    Pass,
}

impl CardDragSelection {
    fn contains(&self, index: usize) -> bool {
        self.active
            && (self.anchor.min(self.current)..=self.anchor.max(self.current)).contains(&index)
    }
}

impl ShengjiCardDragSelection {
    fn contains(&self, index: usize) -> bool {
        self.active
            && (self.anchor.min(self.current)..=self.anchor.max(self.current)).contains(&index)
    }
}

impl Default for UiZoom {
    fn default() -> Self {
        Self { manual: 1.0 }
    }
}

#[derive(Resource, Default)]
struct UiAssets {
    font: Handle<Font>,
    cards: HashMap<(Rank, Suit), Handle<Image>>,
    card_back: Handle<Image>,
    uno_cards: HashMap<(Option<UnoColor>, UnoFace), Handle<Image>>,
    uno_card_back: Handle<Image>,
    table_felt: Handle<Image>,
    primary_button: Handle<Image>,
    secondary_button: Handle<Image>,
    warning_button: Handle<Image>,
    danger_button: Handle<Image>,
    disabled_button: Handle<Image>,
    panel_window: Handle<Image>,
    panel_section: Handle<Image>,
    panel_popup: Handle<Image>,
    player_panel_wide: Handle<Image>,
    player_panel_compact: Handle<Image>,
    poker_chips: HashMap<u16, Handle<Image>>,
    interaction_images: HashMap<(PlayerInteractionKind, bool), Handle<Image>>,
    interaction_sounds: HashMap<(PlayerInteractionKind, u8), Handle<AudioSource>>,
    interaction_cooldown_masks: Vec<Handle<Image>>,
    deal_sounds: Vec<Handle<AudioSource>>,
    place_sounds: Vec<Handle<AudioSource>>,
    shove_sounds: Vec<Handle<AudioSource>>,
    button_click_sounds: Vec<Handle<AudioSource>>,
    error_popup_sound: Handle<AudioSource>,
    bomb_explosion_sound: Handle<AudioSource>,
    summary_score_sound: Handle<AudioSource>,
    summary_die_sound: Handle<AudioSource>,
    quick_voice_sounds: Vec<Handle<AudioSource>>,
    chat_emojis: Vec<Handle<Image>>,
    chat_emoji_icon: Handle<Image>,
    texas_sounds: TexasSoundAssets,
    uno_sounds: UnoSoundAssets,
    sequence_airplane: Handle<Image>,
    shengji_target: Handle<Image>,
    shengji_dart: Handle<Image>,
    chat_open_icon: Handle<Image>,
    chat_close_icon: Handle<Image>,
    quick_voice_icon: Handle<Image>,
    robot_icon: Handle<Image>,
    host_crown: Handle<Image>,
    github_mark: Handle<Image>,
}

const CHAT_EMOJI_ASSET_PATHS: [&str; 30] = [
    "ui/fluent-emoji/laugh.png",
    "ui/fluent-emoji/angry.png",
    "ui/fluent-emoji/surprised.png",
    "ui/fluent-emoji/pleading.png",
    "ui/fluent-emoji/party.png",
    "ui/fluent-emoji/heart.png",
    "ui/fluent-emoji/grinning.png",
    "ui/fluent-emoji/rolling-laugh.png",
    "ui/fluent-emoji/smile.png",
    "ui/fluent-emoji/wink.png",
    "ui/fluent-emoji/heart-eyes.png",
    "ui/fluent-emoji/hearts-face.png",
    "ui/fluent-emoji/kiss.png",
    "ui/fluent-emoji/sunglasses.png",
    "ui/fluent-emoji/star-struck.png",
    "ui/fluent-emoji/cry.png",
    "ui/fluent-emoji/loud-cry.png",
    "ui/fluent-emoji/angry-horns.png",
    "ui/fluent-emoji/flushed.png",
    "ui/fluent-emoji/thinking.png",
    "ui/fluent-emoji/rolling-eyes.png",
    "ui/fluent-emoji/unamused.png",
    "ui/fluent-emoji/expressionless.png",
    "ui/fluent-emoji/tongue.png",
    "ui/fluent-emoji/fearful.png",
    "ui/fluent-emoji/fire.png",
    "ui/fluent-emoji/sparkling-heart.png",
    "ui/fluent-emoji/thumbs-up.png",
    "ui/fluent-emoji/clap.png",
    "ui/fluent-emoji/hundred.png",
];

impl UiAssets {
    fn chat_emoji(&self, emoji: ChatEmoji) -> Handle<Image> {
        self.chat_emojis
            .get(emoji.index())
            .cloned()
            .unwrap_or_default()
    }
}

#[derive(Resource, Default)]
struct AvatarImages {
    remote: HashMap<AvatarId, Handle<Image>>,
    local_png: Option<Vec<u8>>,
    local: Option<Handle<Image>>,
}

#[derive(Resource, Default)]
struct AvatarPicker {
    pending: Option<AvatarPickerReceiver>,
}

#[derive(Resource, Default)]
struct TableFeltPicker {
    pending: Option<TableFeltPickerReceiver>,
}

#[derive(Resource, Default)]
struct TableAppearance {
    loaded_path: Option<PathBuf>,
    custom_felt: Option<Handle<Image>>,
    error: Option<String>,
}

type AvatarPickerResult = Result<Option<PathBuf>, String>;
type AvatarPickerReceiver = Mutex<Receiver<AvatarPickerResult>>;
type TableFeltPickerResult = Result<Option<PathBuf>, String>;
type TableFeltPickerReceiver = Mutex<Receiver<TableFeltPickerResult>>;

#[derive(Component)]
struct UiRoot;

#[derive(Clone)]
struct LobbySeatTransitionSnapshot {
    player: PlayerId,
    center_global: Vec2,
    size: Vec2,
}

#[derive(Resource, Default)]
struct StartGameSeatTransition {
    match_id: Option<MatchId>,
    elapsed: f32,
    seats: Vec<LobbySeatTransitionSnapshot>,
}

impl StartGameSeatTransition {
    fn begin(&mut self, match_id: MatchId, seats: Vec<LobbySeatTransitionSnapshot>) {
        if seats.is_empty() {
            self.clear();
            return;
        }
        self.match_id = Some(match_id);
        self.elapsed = 0.0;
        self.seats = seats;
    }

    fn is_active_for(&self, match_id: MatchId) -> bool {
        self.match_id == Some(match_id) && !self.seats.is_empty()
    }

    fn clear(&mut self) {
        *self = Self::default();
    }
}

#[derive(Clone, Copy, Component)]
struct LobbySeatTransitionSource(PlayerId);

#[derive(Component)]
struct LobbySeatHover {
    seat: u8,
    amount: f32,
}

#[derive(Component)]
struct LobbySeatVisual(u8);

#[derive(Component)]
struct LobbyEmptySeatRing(u8);

#[derive(Component)]
struct LobbyEmptySeatLabel(u8);

#[derive(Clone, Copy, Component)]
struct GameSeatTransitionTarget(PlayerId);

#[derive(Clone, Copy, Component, Default)]
struct GameSeatTransitionPose {
    start_translation: Vec2,
    start_scale: Vec2,
    initialized: bool,
}

#[derive(Component)]
struct PlaySelectionCount;

#[derive(Component)]
struct NoLegalResponseHint;

#[derive(Component)]
struct AutoPlayOverlay;

#[derive(Component)]
struct HandCardSelectionOverlay {
    index: usize,
}

#[derive(Component)]
struct TurnClock;

#[derive(Component)]
struct TurnClockHand;

#[derive(Component)]
struct TurnClockLabel;

#[derive(Component)]
struct RuleHelp {
    tooltip: Entity,
}

#[derive(Component)]
struct TableBackground;

#[derive(Component)]
struct TableAppearanceSlider(TableAppearanceSetting);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TableAppearanceSetting {
    Brightness,
    Vignette,
    Volume,
}

#[derive(Component)]
struct TableAppearanceIndicator {
    setting: TableAppearanceSetting,
    part: TableAppearanceIndicatorPart,
}

#[derive(Clone, Copy)]
enum TableAppearanceIndicatorPart {
    Fill,
    Knob,
}

#[derive(Component)]
struct TableAppearanceLabel(TableAppearanceSetting);

#[derive(Component)]
struct ChatPanel;

#[derive(Component)]
struct ChatToggleIcon;

#[derive(Component)]
struct ChatHistoryText;

#[derive(Component)]
struct ChatInputText;

#[derive(Component)]
struct EmojiMenu;

#[derive(Component)]
struct EmojiScroll;

#[derive(Component)]
struct DeveloperHandInputText;

#[derive(Component)]
struct DeveloperHandInputField;

#[derive(Component)]
struct QuickVoiceMenu;

#[derive(Component)]
struct QuickVoiceScroll;

#[derive(Component)]
struct PlayErrorPopup;

#[derive(Component)]
struct PlayErrorPopupText;

#[derive(Component)]
struct AnimatedSummaryScore {
    target: u32,
    delay: f32,
}

#[derive(Component)]
struct GameSummaryModal;

#[derive(Component)]
struct GameSummaryRow {
    delay: f32,
}

#[derive(Component)]
struct GameSummaryDivider {
    delay: f32,
}

#[derive(Component, Clone, Copy)]
struct TexasRaiseAdjustButton {
    direction: i8,
    step: u32,
    minimum: u32,
    maximum: u32,
}

#[derive(Component)]
struct TexasPotDivider {
    old_layout: bool,
    elapsed: f32,
}

#[derive(Component)]
struct TexasPotHover {
    eligible: Vec<PlayerId>,
}

#[derive(Component, Clone, Copy)]
struct TexasPlayerPanel {
    player: PlayerId,
    base_border: Color,
}

#[derive(Component)]
struct GameSummaryActions {
    delay: f32,
}

#[derive(Component)]
struct AnimatedSummaryText {
    color: Color,
    delay: f32,
}

#[derive(Component)]
struct PlayEffectRoot;

#[derive(Component)]
struct PlayerInteractionLayer;

#[derive(Component)]
struct FinishedHandScoreSource(PlayerId);

#[derive(Component)]
struct ShengjiScoreTrayAnchor;

#[derive(Component)]
struct ShengjiCollectingScoreText;

#[derive(Component)]
struct ShengjiKittyRevealCard {
    index: usize,
}

#[derive(Component)]
struct ShengjiKittyScoreAnchor;

#[derive(Component)]
struct ShengjiKittyScoreText {
    base: u32,
    awarded: u32,
}

#[derive(Component)]
struct ShengjiKittyMultiplier;

#[derive(Component)]
struct ShengjiSettlementTotalAnchor;

#[derive(Component)]
struct ShengjiSettlementTotalText {
    target: u32,
}

#[derive(Component)]
struct ShengjiSettlementModal;

#[derive(Component)]
struct ShengjiSettlementRow {
    delay: f32,
}

#[derive(Component)]
struct ShengjiSettlementActions {
    delay: f32,
}

#[derive(Component)]
struct ShengjiSettlementOutcomeText;

#[derive(Component)]
struct ShengjiFailedThrowCard {
    index: usize,
    count: usize,
    stage: ShengjiThrowFailureStage,
    direction: Vec2,
    elapsed: f32,
}

#[derive(Component)]
struct ShengjiFailedThrowLabel {
    returning: bool,
    elapsed: f32,
}

#[derive(Component)]
struct ShengjiThrowPenaltyFloat {
    source: Vec2,
    target: Vec2,
    elapsed: f32,
}

#[derive(Component)]
struct ShengjiThrowPenaltyScorePulse {
    elapsed: f32,
}

#[derive(Component)]
struct ShengjiDealerBadge;

#[derive(Component)]
struct ShengjiLevelIndicator {
    base_color: Color,
}

#[derive(Component)]
struct ShengjiTimedReveal {
    delay: f32,
}

#[derive(Component)]
struct ActiveShengjiScoreAbsorb {
    source: Vec2,
    target: Vec2,
    elapsed: f32,
    delay: f32,
    curve: f32,
}

#[derive(Component)]
struct ActiveScoreCaptureCard {
    source: Vec2,
    target: Vec2,
    elapsed: f32,
    delay: f32,
    curve: f32,
}

#[derive(Component)]
struct ActiveScoreVortex {
    elapsed: f32,
}

#[derive(Component)]
struct ActiveScoreGainText {
    elapsed: f32,
    text: Entity,
}

/// 玩家框相对于牌桌的方位；两个游戏和公共弹窗布局共同使用。
#[derive(Clone, Copy)]
enum SeatSide {
    Left,
    Top,
    Right,
}

#[derive(Clone, Copy, Component)]
enum PlayerGameScoreText {
    Opponent { player: PlayerId, side: SeatSide },
    Own(PlayerId),
}

impl PlayerGameScoreText {
    fn player(self) -> PlayerId {
        match self {
            Self::Opponent { player, .. } | Self::Own(player) => player,
        }
    }

    fn label(self, score: u32) -> String {
        match self {
            Self::Opponent { .. } | Self::Own(_) => score.to_string(),
        }
    }

    fn gain_left(self, center_x: f32, width: f32) -> f32 {
        match self {
            Self::Opponent {
                side: SeatSide::Right,
                ..
            } => center_x - width * 0.5 - 84.0,
            Self::Opponent { .. } | Self::Own(_) => center_x + width * 0.5 + 8.0,
        }
    }
}

#[derive(Component)]
struct OpponentBadge {
    player: PlayerId,
    score_popup: Option<Entity>,
    interaction_menu: Entity,
}

#[derive(Component)]
struct PlayerAvatarAnchor(PlayerId);

#[derive(Component)]
struct UnoSwapTargetPanel {
    selected: bool,
}

#[derive(Component)]
struct UnoExpansionStatus;

#[derive(Component)]
struct UnoExpansionStatusFrame;

#[derive(Component)]
struct AutoPlayRobotIndicator;

#[derive(Clone, Copy, Component)]
struct AutoPlayAntennaLight {
    player: PlayerId,
    part: AutoPlayAntennaLightPart,
}

#[derive(Clone, Copy)]
enum AutoPlayAntennaLightPart {
    Glow,
    Ray,
}

#[derive(Component)]
struct ActiveChatBubble {
    player: PlayerId,
    text: Option<Entity>,
    emoji_image: Option<Entity>,
    emoji: bool,
    width: f32,
    elapsed: f32,
    duration: f32,
}

#[derive(Component)]
struct ChatBubbleText;

#[derive(Component)]
struct InteractionMenuPanel(PlayerId);

#[derive(Component)]
struct InteractionCooldownMask {
    player: PlayerId,
    kind: PlayerInteractionKind,
}

#[derive(Component)]
struct ActivePlayerInteraction {
    source: Vec2,
    target: Vec2,
    kind: PlayerInteractionKind,
    sound_variant: u8,
    play_sound: bool,
    elapsed: f32,
    appear_duration: f32,
    travel_duration: f32,
    impact_duration: f32,
    launched: bool,
    impacted: bool,
}

#[derive(Component)]
struct SequenceEffectCard {
    index: usize,
}

#[derive(Component)]
struct SequenceGuideSegment {
    index: usize,
    count: usize,
}

#[derive(Component)]
struct SequenceEffectLabel;

#[derive(Component, Clone, Copy)]
struct SequenceEffectLabelPart {
    outline: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SequenceEffectMotif {
    Wind,
    Flower,
    Airplane,
}

#[derive(Component)]
struct SequenceWindStreak {
    index: usize,
}

#[derive(Component)]
struct SequenceFlowerPart {
    index: usize,
    petal: bool,
}

#[derive(Component)]
struct SequenceAirplane;

#[derive(Component)]
struct SequenceAirplaneTrail {
    index: usize,
}

#[derive(Component)]
struct BombEffectBody {
    source: Vec2,
}

#[derive(Component)]
struct BombFuseSpark;

#[derive(Component)]
struct BombExplosionFlash;

#[derive(Component)]
struct BombExplosionRing;

#[derive(Component)]
struct BombExplosionParticle {
    direction: Vec2,
    distance: f32,
}

#[derive(Component)]
struct HeavenBombBackdrop;

#[derive(Component)]
struct HeavenBombFlash;

#[derive(Component)]
struct HeavenBombRay {
    index: usize,
}

#[derive(Component)]
struct HeavenBombShockRing {
    delay: f32,
}

#[derive(Component)]
struct HeavenBombParticle {
    direction: Vec2,
    distance: f32,
    delay: f32,
}

#[derive(Component)]
struct HeavenBombTitle;

#[derive(Component)]
struct HeavenBombTitleText;

#[derive(Clone, Component)]
enum UiAction {
    FocusInput(InputField),
    OpenHostGamePicker,
    CloseHostGamePicker,
    CreateRoom(GameKind),
    JoinRoom,
    ChooseAvatar,
    ClearAvatar,
    ToggleProfile,
    OpenPlayerProfile(PlayerProfilePage),
    SelectProfileGameTab(ProfileGameTab),
    ToggleSettings,
    StartUpdate,
    OpenGitHubRepository,
    HideUpdateDialog,
    RestartToUpdate,
    ChooseTableFelt,
    UseDefaultTableFelt,
    SelectSeat(SeatId),
    ToggleReady,
    UpdateRules(RuleSet),
    UpdateTexasRules(TexasHoldemRuleSet),
    UpdateShengjiRules(ShengjiRuleSet),
    UpdateUnoRules(UnoRuleSet),
    ToggleUnoModeMenu,
    CloseUnoModeMenu,
    ToggleUnoExpansionSettings,
    ToggleUnoCard(UnoCard),
    SubmitUnoCard,
    CloseUnoColorChoice,
    UnoChooseInitialColor(UnoColor),
    UnoPlayCard(UnoCard, Option<UnoColor>),
    UnoJumpIn(UnoCard),
    ToggleUnoSwapTarget(PlayerId),
    ConfirmUnoSwapTargets,
    UnoDrawCard,
    UnoPassAfterDraw,
    UnoAcceptDrawPenalty,
    UnoChallengeDrawFour,
    UnoResolveSkip,
    UnoCall,
    UnoReport(PlayerId),
    SetTexasRaiseTo(u32),
    TexasAct(TexasHoldemAction),
    ShengjiDeclare(Vec<ShengjiCard>),
    ConfirmShengjiBidPass,
    ShengjiBottomCopy(Vec<ShengjiCard>),
    DeclineBottomCopy,
    ToggleShengjiCard,
    ShengjiHint,
    ShowShengjiPreviousTrick,
    ToggleShengjiBuried,
    SubmitShengjiCards,
    DeclineFiveTrumpCrossing,
    StartGame,
    ReturnToLobby,
    PlayAgain,
    LeaveRoom,
    #[cfg(feature = "developer")]
    FocusDeveloperHand,
    ToggleInteractionMenu(PlayerId),
    SendInteraction {
        target: PlayerId,
        kind: PlayerInteractionKind,
    },
    ToggleChatPanel,
    ToggleAutoPlay,
    FocusChatInput,
    ToggleQuickVoiceMenu,
    ToggleEmojiMenu,
    SendQuickVoice(u8),
    SendEmoji(ChatEmoji),
    Hint,
    ToggleCard,
    Play,
    Pass,
}

#[derive(Component)]
struct ButtonTint {
    normal: Color,
    hovered: Color,
    pressed: Color,
}

#[derive(Component)]
struct BackgroundButtonTint;

#[derive(Component)]
struct HandCardVisual {
    button: Entity,
    card: Card,
    index: usize,
    selected: bool,
    hover_amount: f32,
    selected_amount: f32,
    deal_elapsed: f32,
    dealing: bool,
    hand_len: usize,
}

#[derive(Component)]
struct HandCardSlot {
    card: Card,
    index: usize,
    is_last: bool,
    hover_amount: f32,
}

#[derive(Component)]
struct ShengjiHandCardSlot {
    card: ShengjiCard,
    index: usize,
    hand_len: usize,
    is_last: bool,
    hover_amount: f32,
}

#[derive(Component)]
struct ShengjiHandCardVisual {
    button: Entity,
    card: ShengjiCard,
    index: usize,
    selected: bool,
    hover_amount: f32,
    selected_amount: f32,
    deal_elapsed: f32,
    dealing: bool,
    hand_len: usize,
}

#[derive(Component)]
struct UnoHandCardVisual {
    button: Entity,
    card: UnoCard,
    selected: bool,
    hover_amount: f32,
    selected_amount: f32,
}

#[derive(Component)]
struct UnoHandCardButton;

#[derive(Component)]
struct UnoExtensionCardHelp {
    title: &'static str,
    description: &'static str,
}

#[derive(Component)]
struct UnoExtensionCardHelpOverlay {
    title: Entity,
    description: Entity,
}

#[derive(Resource, Default)]
struct UnoPresentationState {
    events: VecDeque<UnoEvent>,
    last_snapshot: Option<UnoSnapshot>,
}

#[derive(Component)]
struct UnoDrawPileAnchor;

#[derive(Component)]
struct UnoDiscardPileAnchor;

#[derive(Component)]
struct UnoDiscardCard(UnoCard);

#[derive(Component, Clone, Copy)]
enum UnoFlipTarget {
    Own(usize),
    Opponent { player: PlayerId, index: usize },
    DrawPile(usize),
    DiscardPile(usize),
}

#[derive(Component)]
struct UnoFlyingCard {
    elapsed: f32,
    delay: f32,
    start: Vec2,
    staging: Vec2,
    control: Vec2,
    target: Vec2,
    duration: f32,
    draw_animation: bool,
    played_card: Option<UnoCard>,
    start_angle: f32,
    end_angle: f32,
}

#[derive(Component)]
struct UnoFlipOverlay {
    elapsed: f32,
}

#[derive(Component)]
struct UnoFlipCard {
    elapsed: f32,
    delay: f32,
    old_face: Handle<Image>,
    new_face: Handle<Image>,
    swapped: bool,
    base_transform: UiTransform,
    pile: bool,
}

#[derive(Component)]
struct UnoPaletteEffect {
    elapsed: f32,
}

#[derive(Component)]
struct UnoPaletteSelectedSector {
    elapsed: f32,
}

#[derive(Component)]
struct UnoPaletteColorRing {
    elapsed: f32,
    delay: f32,
    color: Color,
    start_scale: f32,
    end_scale: f32,
    max_alpha: f32,
}

#[derive(Component)]
struct UnoPaletteParticle {
    elapsed: f32,
    delay: f32,
    origin: Vec2,
    direction: Vec2,
    size: Vec2,
    color: Color,
    rotation: f32,
}

#[derive(Component)]
struct UnoReverseArrow {
    elapsed: f32,
    delay: f32,
    color: Color,
    max_alpha: f32,
    shadow_alpha: f32,
}

#[derive(Component)]
struct ShengjiHandCardSelectionOverlay {
    index: usize,
}

#[derive(Clone, Copy)]
struct HandCardPose {
    translation: Val2,
    rotation: Rot2,
}

#[derive(Clone, Copy)]
enum ButtonKind {
    Primary,
    Secondary,
    Warning,
    Pass,
}

#[derive(Clone, Copy)]
enum PanelSkin {
    Window,
    Section,
    Popup,
}

#[derive(Component)]
struct GameSummaryPanelTexture;

#[derive(Component)]
struct UnoModeDropdownPanel;

#[derive(Component)]
struct ShengjiSettlementPanelTexture;

type ButtonInteractions<'w, 's> =
    Query<'w, 's, (&'static Interaction, &'static UiAction), (Changed<Interaction>, With<Button>)>;

type HandCardAnimations<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut HandCardVisual,
        &'static mut UiTransform,
        &'static mut Outline,
        &'static mut BoxShadow,
        &'static mut ImageNode,
        &'static mut BorderColor,
    ),
>;

#[derive(SystemParam)]
struct VisualAssets<'w> {
    ui: Res<'w, UiAssets>,
    avatars: Res<'w, AvatarImages>,
    table: Res<'w, TableAppearance>,
    play_error: Res<'w, PlayErrorToast>,
    game_summary: Res<'w, GameSummaryAnimation>,
    play_effect: Res<'w, PlayEffectState>,
    score_capture: Res<'w, ScoreCaptureEffectState>,
    shengji_score_capture: Res<'w, ShengjiScoreCaptureEffectState>,
    shengji_settlement: Res<'w, ShengjiSettlementAnimation>,
    shengji_presentation: Res<'w, ShengjiPresentationState>,
    start_game_transition: Res<'w, StartGameSeatTransition>,
    texas_chips: Res<'w, TexasChipTableState>,
    updater: Res<'w, UpdateManager>,
}

#[derive(SystemParam)]
struct LocalUiResources<'w> {
    avatar_images: ResMut<'w, AvatarImages>,
    avatar_picker: ResMut<'w, AvatarPicker>,
    table_felt_picker: ResMut<'w, TableFeltPicker>,
    table_appearance: ResMut<'w, TableAppearance>,
}

mod animation;
mod appearance;
mod audio;
mod bootstrap;
mod chat_view;
mod controller;
mod embedded_assets;
mod input;
mod interaction;
mod overlays;
mod profile_stats;
mod qigui523;
mod screens;
mod seat_transition;
mod shengji;
mod texas_holdem;
mod turn_border;
mod uno;
mod uno_audio;
mod update;
mod widgets;

use animation::*;
use appearance::*;
use audio::*;
use bootstrap::*;
use chat_view::*;
use controller::*;
use embedded_assets::*;
use input::*;
use interaction::*;
use overlays::*;
use profile_stats::*;
use qigui523::*;
use screens::*;
use seat_transition::*;
use shengji::*;
use texas_holdem::*;
use turn_border::*;
use uno::*;
use uno_audio::*;
use update::*;
use widgets::*;

#[cfg(test)]
mod tests;

pub(crate) fn run() {
    let mut form = ConnectionForm::default();
    let profile = match LocalPlayerProfile::load_or_create() {
        Ok(profile) => profile,
        Err(error) => {
            form.error = Some(error);
            LocalPlayerProfile {
                identity: PlayerIdentity::generate()
                    .expect("the operating system provides entropy"),
                rating: PlayerRatingProfile {
                    reference_points: 0,
                    completed_games: 0,
                    applied_matches: HashSet::new(),
                    last_change: None,
                },
                game_profiles: PlayerGameProfiles::default(),
            }
        }
    };
    let audio_volume = form.audio_volume;
    let mut app = App::new();
    configure_runtime_asset_source(&mut app);
    app.insert_resource(ClearColor(TABLE_BG))
        .insert_resource(form)
        .insert_resource(GlobalVolume::new(Volume::Linear(audio_volume)))
        .insert_resource(profile)
        .insert_resource(AvatarImages::default())
        .insert_resource(AvatarPicker::default())
        .insert_resource(TableFeltPicker::default())
        .insert_resource(TableAppearance::default())
        .insert_resource(PlayErrorToast::default())
        .insert_resource(GameSummaryAnimation::default())
        .insert_resource(TexasRaiseHoldState::default())
        .insert_resource(PlayEffectState::default())
        .insert_resource(ScoreCaptureEffectState::default())
        .insert_resource(ShengjiScoreCaptureEffectState::default())
        .insert_resource(ShengjiSettlementAnimation::default())
        .insert_resource(ShengjiPresentationState::default())
        .insert_resource(UnoPresentationState::default())
        .insert_resource(UnoAudioState::default())
        .insert_resource(StartGameSeatTransition::default())
        .insert_resource(TurnBorderAnimationState::default())
        .insert_resource(PlayerInteractionCooldown::default())
        .insert_resource(UiState {
            dirty: true,
            ..default()
        })
        .insert_resource(UiZoom::default())
        .insert_resource(CardDragSelection::default())
        .insert_resource(ShengjiCardDragSelection::default())
        .insert_resource(ChatPanelState::default())
        .insert_resource(DeveloperHandInput::default())
        .insert_resource(TexasChipTableState::default())
        .insert_resource(UpdateManager::default())
        .add_plugins(
            DefaultPlugins
                .set(LogPlugin {
                    // Parley deliberately uses ICU4X's non-complex-script segmenter.
                    // Chinese text then falls back correctly, but ICU logs a warning for
                    // every layout pass because no CJK dictionary was requested.
                    filter: format!("{DEFAULT_FILTER}icu_provider::error=error"),
                    ..default()
                })
                .set(AssetPlugin {
                    file_path: asset_file_path(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "LeoCard".to_owned(),
                        resolution: (DESIGN_WIDTH as u32, DESIGN_HEIGHT as u32).into(),
                        resize_constraints: WindowResizeConstraints {
                            min_width: 640.0,
                            min_height: 400.0,
                            ..default()
                        },
                        resizable: true,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(UiMaterialPlugin::<TableBackgroundMaterial>::default())
        .add_plugins(UiMaterialPlugin::<TurnBorderMaterial>::default())
        .add_plugins(UiMaterialPlugin::<UnoPaletteMaterial>::default())
        .add_systems(Startup, (setup_camera, load_ui_assets))
        .add_systems(
            Update,
            (
                (
                    update_ui_scale,
                    sync_ime_enabled,
                    handle_text_input,
                    handle_avatar_drop,
                    poll_avatar_picker,
                    poll_table_felt_picker,
                    sync_table_appearance,
                    update_button_tints,
                    play_button_click_sounds,
                    play_uno_card_selection_sounds,
                    animate_button_presses,
                    animate_lobby_seat_hover,
                    handle_lobby_bot_seat_right_click,
                    update_rule_help_tooltips,
                    sync_uno_extension_card_help,
                    handle_table_appearance_sliders,
                    handle_card_drag_selection,
                    animate_hand_card_slots,
                    handle_shengji_card_drag_selection,
                    animate_shengji_hand_card_slots,
                )
                    .chain(),
                (
                    (
                        (animate_hand_cards, animate_uno_hand_cards).chain(),
                        (animate_uno_swap_target_panels, animate_turn_clocks),
                        tick_player_interaction_cooldown,
                        (sync_card_drag_preview, sync_shengji_card_drag_preview).chain(),
                        handle_texas_raise_button_hold,
                        handle_buttons,
                        close_interaction_menu_on_outside_click,
                        sync_opponent_badge_popups,
                        sync_interaction_cooldown_masks,
                        sync_selection_label,
                        sync_developer_hand_input_text,
                        animate_no_legal_response_hint,
                        (
                            poll_network,
                            sync_uno_presentation,
                            sync_shengji_presentation,
                            update_shengji_settlement_animation,
                        )
                            .chain(),
                        (sync_texas_chip_state, play_texas_audio_cues).chain(),
                        sync_chat_messages,
                        sync_player_interactions,
                        (sync_score_capture_effect, sync_shengji_score_capture_effect).chain(),
                        animate_player_interactions,
                        (
                            animate_score_capture_effects,
                            animate_shengji_score_capture_score,
                        )
                            .chain(),
                        animate_chat_bubbles,
                    )
                        .chain(),
                    (
                        sync_turn_timer_label,
                        sync_play_error_toast,
                        animate_play_error_popup,
                        sync_play_effect,
                        advance_play_effect,
                        animate_sequence_play_effect,
                        animate_bomb_play_effect,
                        animate_heaven_bomb_play_effect,
                        update_summary_animation,
                        animate_game_summary_visuals,
                        animate_summary_scores,
                        (
                            queue_deal_animations,
                            queue_shengji_deal_animations,
                            animate_shengji_hand_cards,
                        )
                            .chain(),
                        (
                            animate_texas_deal_cards,
                            animate_texas_flying_card_backs,
                            animate_texas_board_card_backs,
                            animate_texas_board_card_flips,
                            animate_texas_showdown_reveal,
                            animate_texas_chip_sprites,
                            animate_texas_pot_dividers,
                        )
                            .chain(),
                        (
                            highlight_texas_pot_eligible_players,
                            sync_texas_own_fold_tooltip,
                            animate_texas_action_feedback,
                        )
                            .chain(),
                        play_pending_deal_sounds,
                        animate_chat_panel,
                        animate_auto_play_robot_indicators,
                        sync_chat_panel_text,
                        scroll_chat_menus,
                        (
                            sync_avatar_images,
                            poll_update_events,
                            render_ui,
                            spawn_uno_presentation_effects,
                            play_uno_audio_cues,
                            (
                                // 飞牌结束时先应用 despawn，再在同一帧显示权威弃牌；
                                // 顺序反过来会让两者同时缺席一个渲染帧，产生落地闪烁。
                                (animate_uno_flying_cards, sync_uno_discard_reveal).chain(),
                                animate_uno_palette_effects,
                                animate_uno_palette_selected_sectors,
                                animate_uno_palette_color_rings,
                                animate_uno_palette_particles,
                                animate_uno_reverse_effects,
                                animate_uno_flip_effects,
                            ),
                            sync_update_dialog,
                            sync_shengji_bidding_countdown,
                            (
                                animate_shengji_failed_throw_cards,
                                animate_shengji_failed_throw_labels,
                                animate_shengji_throw_penalty_floats,
                                animate_shengji_throw_penalty_score_pulses,
                            )
                                .chain(),
                            animate_shengji_settlement_visuals,
                            advance_shengji_presentation,
                            animate_shengji_presentation,
                            animate_shengji_bottom_flip_markers,
                            animate_shengji_power_outage_markers,
                            play_shengji_audio_cues,
                            spawn_shengji_settlement_absorption,
                            animate_shengji_score_absorbs,
                            animate_start_game_seat_transition,
                            animate_turn_border_traces,
                        )
                            .chain(),
                    )
                        .chain(),
                )
                    .chain(),
            )
                .chain(),
        )
        .run();
}
