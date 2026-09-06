//! 已加载 UI 资源、头像缓存与文件选择器状态。

use leocard_mahjong::MahjongTileKind;
use leocard_protocol::{AvatarId, ChatEmoji, PlayerInteractionKind};
use leocard_qigui523::{QiGuiRank, QiGuiSuit};
use leocard_uno::{UnoColor, UnoFace};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::mpsc::Receiver;

use super::*;

pub const UI_FONT_ASSET: &str = "fonts/ChillRoundGothic-Medium.ttf";
pub const TABLE_FELT_ASSET: &str = "vendor/opengameart/green-textile/table_felt_dark_green.png";

#[derive(Resource, Default)]
pub struct UiAssets {
    pub font: Handle<Font>,
    pub games: GameVisualAssets,
    pub controls: ControlAssets,
    pub social: SocialAssets,
    pub audio: CommonAudioAssets,
    pub table_felt: Handle<Image>,
}

#[derive(Default)]
pub struct GameVisualAssets {
    pub cards: HashMap<(QiGuiRank, QiGuiSuit), Handle<Image>>,
    pub card_back: Handle<Image>,
    pub uno_cards: HashMap<(Option<UnoColor>, UnoFace), Handle<Image>>,
    pub uno_card_back: Handle<Image>,
    pub mahjong_tiles: HashMap<MahjongTileKind, Handle<Image>>,
    pub mahjong_tile_heights: HashMap<MahjongTileKind, Handle<Image>>,
    pub mahjong_tile_back: Handle<Image>,
    pub mahjong_turn_arrow: Handle<Image>,
    pub poker_chips: HashMap<u16, Handle<Image>>,
    pub texas_sounds: TexasSoundAssets,
    pub uno_sounds: UnoSoundAssets,
    pub sequence_airplane: Handle<Image>,
    pub shengji_target: Handle<Image>,
    pub shengji_dart: Handle<Image>,
}

#[derive(Default)]
pub struct ControlAssets {
    pub primary_button: Handle<Image>,
    pub secondary_button: Handle<Image>,
    pub warning_button: Handle<Image>,
    pub danger_button: Handle<Image>,
    pub disabled_button: Handle<Image>,
    pub panel_window: Handle<Image>,
    pub panel_section: Handle<Image>,
    pub panel_popup: Handle<Image>,
    pub player_panel_wide: Handle<Image>,
    pub player_panel_compact: Handle<Image>,
    pub robot_icon: Handle<Image>,
    pub host_crown: Handle<Image>,
    pub github_mark: Handle<Image>,
}

#[derive(Default)]
pub struct SocialAssets {
    pub interaction_images: HashMap<(PlayerInteractionKind, bool), Handle<Image>>,
    pub interaction_cooldown_masks: Vec<Handle<Image>>,
    pub chat_emojis: Vec<Handle<Image>>,
    pub chat_emoji_icon: Handle<Image>,
    pub chat_open_icon: Handle<Image>,
    pub chat_close_icon: Handle<Image>,
    pub quick_voice_icon: Handle<Image>,
}

#[derive(Default)]
pub struct CommonAudioAssets {
    pub interaction_sounds: HashMap<(PlayerInteractionKind, u8), Handle<AudioSource>>,
    pub deal_sounds: Vec<Handle<AudioSource>>,
    pub place_sounds: Vec<Handle<AudioSource>>,
    pub shove_sounds: Vec<Handle<AudioSource>>,
    pub button_click_sounds: Vec<Handle<AudioSource>>,
    pub error_popup_sound: Handle<AudioSource>,
    pub bomb_explosion_sound: Handle<AudioSource>,
    pub summary_score_sound: Handle<AudioSource>,
    pub summary_die_sound: Handle<AudioSource>,
    pub quick_voice_sounds: Vec<Handle<AudioSource>>,
}

pub const CHAT_EMOJI_ASSET_PATHS: [&str; 30] = [
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
    pub fn chat_emoji(&self, emoji: ChatEmoji) -> Handle<Image> {
        self.social
            .chat_emojis
            .get(emoji.index())
            .cloned()
            .unwrap_or_default()
    }
}

#[derive(Resource, Default)]
pub struct AvatarImages {
    pub remote: HashMap<AvatarId, Handle<Image>>,
    pub local_png: Option<Vec<u8>>,
    pub local: Option<Handle<Image>>,
}

#[derive(Resource, Default)]
pub struct AvatarPicker {
    pub pending: Option<AvatarPickerReceiver>,
}

#[derive(Resource, Default)]
pub struct TableFeltPicker {
    pub pending: Option<TableFeltPickerReceiver>,
}

#[derive(Resource, Default)]
pub struct TableAppearance {
    pub loaded_path: Option<PathBuf>,
    pub custom_felt: Option<Handle<Image>>,
    pub error: Option<String>,
}

pub type AvatarPickerResult = Result<Option<PathBuf>, String>;
pub type AvatarPickerReceiver = Mutex<Receiver<AvatarPickerResult>>;
pub type TableFeltPickerResult = Result<Option<PathBuf>, String>;
pub type TableFeltPickerReceiver = Mutex<Receiver<TableFeltPickerResult>>;
