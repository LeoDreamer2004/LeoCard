//! 已加载 UI 资源、头像缓存与文件选择器状态。

use bevy::prelude::*;
use leocard_protocol::{AvatarId, ChatEmoji, PlayerInteractionKind};
use leocard_qigui523::{QiGuiRank, QiGuiSuit};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::mpsc::Receiver;

pub(crate) const UI_FONT_ASSET: &str = "fonts/ChillRoundGothic-Medium.ttf";
pub(crate) const TABLE_FELT_ASSET: &str =
    "vendor/opengameart/green-textile/table_felt_dark_green.png";

#[derive(Resource, Default)]
pub(crate) struct UiAssets {
    pub font: Handle<Font>,
    pub playing_cards: PlayingCardAssets,
    pub controls: ControlAssets,
    pub home: HomeAssets,
    pub achievements: AchievementAssets,
    pub social: SocialAssets,
    pub audio: CommonAudioAssets,
    pub table_felt: Handle<Image>,
}

#[derive(Default)]
pub(crate) struct HomeAssets {
    pub panel: Handle<Image>,
    pub settings_page: Handle<Image>,
    pub game_card: Handle<Image>,
    pub game_card_hover: Handle<Image>,
    pub warning_toast: Handle<Image>,
    pub rule_left: Handle<Image>,
    pub rule_left_highlighted: Handle<Image>,
    pub rule_right: Handle<Image>,
    pub rule_right_highlighted: Handle<Image>,
    pub help_question: Handle<Image>,
    pub checkbox: Handle<Image>,
    pub checkbox_highlighted: Handle<Image>,
    pub checkbox_selected: Handle<Image>,
    pub checkbox_selected_highlighted: Handle<Image>,
    pub button: Handle<Image>,
    pub danger_button: Handle<Image>,
    pub danger_button_hover: Handle<Image>,
    pub cool_button: Handle<Image>,
    pub cool_button_hover: Handle<Image>,
    pub purple_button: Handle<Image>,
    pub purple_button_compact: Handle<Image>,
    pub button_arrows: Handle<Image>,
    pub close_button: Handle<Image>,
    pub close_button_highlighted: Handle<Image>,
    pub settings_icon: Handle<Image>,
    pub exit_icon: Handle<Image>,
    pub slider: Handle<Image>,
    pub slider_highlighted: Handle<Image>,
    pub slider_handle: Handle<Image>,
    pub slider_handle_highlighted: Handle<Image>,
    pub input: Handle<Image>,
    pub focused_input: Handle<Image>,
    pub game_art: [Handle<Image>; 5],
    pub reference_level_icons: [Handle<Image>; 10],
}

#[derive(Default)]
pub(crate) struct AchievementAssets {
    pub icon: Handle<Image>,
    pub emblems: [Handle<Image>; 6],
    pub medals: [Handle<Image>; 3],
    pub scroll_arrow: Handle<Image>,
    pub toast: Handle<Image>,
    pub toast_gold: Handle<Image>,
    pub sound: Handle<AudioSource>,
    pub gold_sound: Handle<AudioSource>,
}

#[derive(Default)]
pub(crate) struct PlayingCardAssets {
    pub cards: HashMap<(QiGuiRank, QiGuiSuit), Handle<Image>>,
    pub card_back: Handle<Image>,
}

#[derive(Default)]
pub(crate) struct ControlAssets {
    pub game_play_button: Handle<Image>,
    pub game_play_button_hover: Handle<Image>,
    pub game_pass_button: Handle<Image>,
    pub game_pass_button_hover: Handle<Image>,
    pub game_hint_button: Handle<Image>,
    pub game_hint_button_hover: Handle<Image>,
    pub game_warning_button: Handle<Image>,
    pub game_warning_button_hover: Handle<Image>,
    pub robot_icon: Handle<Image>,
    pub host_crown: Handle<Image>,
    pub github_mark: Handle<Image>,
}

#[derive(Default)]
pub(crate) struct SocialAssets {
    pub interaction_images: HashMap<(PlayerInteractionKind, bool), Handle<Image>>,
    pub interaction_cooldown_masks: Vec<Handle<Image>>,
    pub chat_emojis: Vec<Handle<Image>>,
    pub chat_emoji_icon: Handle<Image>,
    pub quick_voice_icon: Handle<Image>,
}

#[derive(Default)]
pub(crate) struct CommonAudioAssets {
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

pub(super) const CHAT_EMOJI_ASSET_PATHS: [&str; ChatEmoji::ALL.len()] = [
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
    "ui/tieba-emoji/image_emoticon.png",
    "ui/tieba-emoji/image_emoticon2.png",
    "ui/tieba-emoji/image_emoticon3.png",
    "ui/tieba-emoji/image_emoticon4.png",
    "ui/tieba-emoji/image_emoticon5.png",
    "ui/tieba-emoji/image_emoticon6.png",
    "ui/tieba-emoji/image_emoticon7.png",
    "ui/tieba-emoji/image_emoticon8.png",
    "ui/tieba-emoji/image_emoticon9.png",
    "ui/tieba-emoji/image_emoticon10.png",
    "ui/tieba-emoji/image_emoticon11.png",
    "ui/tieba-emoji/image_emoticon12.png",
    "ui/tieba-emoji/image_emoticon13.png",
    "ui/tieba-emoji/image_emoticon14.png",
    "ui/tieba-emoji/image_emoticon15.png",
    "ui/tieba-emoji/image_emoticon16.png",
    "ui/tieba-emoji/image_emoticon17.png",
    "ui/tieba-emoji/image_emoticon18.png",
    "ui/tieba-emoji/image_emoticon19.png",
    "ui/tieba-emoji/image_emoticon20.png",
    "ui/tieba-emoji/image_emoticon21.png",
    "ui/tieba-emoji/image_emoticon22.png",
    "ui/tieba-emoji/image_emoticon23.png",
    "ui/tieba-emoji/image_emoticon24.png",
    "ui/tieba-emoji/image_emoticon25.png",
    "ui/tieba-emoji/image_emoticon26.png",
    "ui/tieba-emoji/image_emoticon27.png",
    "ui/tieba-emoji/image_emoticon28.png",
    "ui/tieba-emoji/image_emoticon29.png",
    "ui/tieba-emoji/image_emoticon30.png",
    "ui/tieba-emoji/image_emoticon31.png",
    "ui/tieba-emoji/image_emoticon32.png",
    "ui/tieba-emoji/image_emoticon33.png",
    "ui/tieba-emoji/image_emoticon34.png",
    "ui/tieba-emoji/image_emoticon35.png",
    "ui/tieba-emoji/image_emoticon36.png",
    "ui/tieba-emoji/image_emoticon37.png",
    "ui/tieba-emoji/image_emoticon38.png",
    "ui/tieba-emoji/image_emoticon39.png",
    "ui/tieba-emoji/image_emoticon40.png",
    "ui/tieba-emoji/image_emoticon41.png",
    "ui/tieba-emoji/image_emoticon42.png",
    "ui/tieba-emoji/image_emoticon43.png",
    "ui/tieba-emoji/image_emoticon44.png",
    "ui/tieba-emoji/image_emoticon45.png",
    "ui/tieba-emoji/image_emoticon46.png",
    "ui/tieba-emoji/image_emoticon47.png",
    "ui/tieba-emoji/image_emoticon48.png",
    "ui/tieba-emoji/image_emoticon49.png",
    "ui/tieba-emoji/image_emoticon50.png",
];

impl UiAssets {
    pub(crate) fn chat_emoji(&self, emoji: ChatEmoji) -> Handle<Image> {
        self.social
            .chat_emojis
            .get(emoji.index())
            .cloned()
            .unwrap_or_default()
    }
}

#[derive(Resource, Default)]
pub(crate) struct AvatarImages {
    pub remote: HashMap<AvatarId, Handle<Image>>,
    pub local_png: Option<Vec<u8>>,
    pub local: Option<Handle<Image>>,
}

#[derive(Resource, Default)]
pub(crate) struct AvatarPicker {
    pub pending: Option<AvatarPickerReceiver>,
}

#[derive(Resource, Default)]
pub(crate) struct TableFeltPicker {
    pub pending: Option<TableFeltPickerReceiver>,
}

#[derive(Resource, Default)]
pub(crate) struct TableAppearance {
    pub loaded_path: Option<PathBuf>,
    pub custom_felt: Option<Handle<Image>>,
    pub error: Option<String>,
}

type AvatarPickerResult = Result<Option<PathBuf>, String>;
pub(crate) type AvatarPickerReceiver = Mutex<Receiver<AvatarPickerResult>>;
type TableFeltPickerResult = Result<Option<PathBuf>, String>;
pub(crate) type TableFeltPickerReceiver = Mutex<Receiver<TableFeltPickerResult>>;
