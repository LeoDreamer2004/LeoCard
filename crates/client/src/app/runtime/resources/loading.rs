//! Bevy 场景基础节点与静态资源的统一加载。

use super::{
    CHAT_EMOJI_ASSET_PATHS, CommonAudioAssets, ControlAssets, HomeAssets, PlayingCardAssets,
    SocialAssets, TABLE_FELT_ASSET, UI_FONT_ASSET, UiAssets, card_asset_path,
    interaction_cooldown_mask_image,
};
use crate::app::shell::{INTERACTION_COOLDOWN_MASK_FRAMES, PlayerInteractionLayer};
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use leocard_protocol::{PlayerInteractionKind, QUICK_VOICE_COUNT};
use leocard_qigui523::build_deck;
use std::collections::HashMap;

pub(crate) fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn((
        PlayerInteractionLayer,
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            right: px(0),
            top: px(0),
            bottom: px(0),
            ..default()
        },
        GlobalZIndex(1100),
        FocusPolicy::Pass,
    ));
}

pub(crate) fn load_ui_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
) {
    let mut cards = HashMap::new();
    for card in build_deck(1) {
        cards.entry((card.rank(), card.suit())).or_insert_with(|| {
            asset_server.load::<Image>(card_asset_path(card.rank(), card.suit()))
        });
    }
    let mut interaction_images = HashMap::new();
    let mut interaction_sounds = HashMap::new();
    for (kind, name) in [
        (PlayerInteractionKind::Flower, "flower"),
        (PlayerInteractionKind::Egg, "egg"),
        (PlayerInteractionKind::Wine, "wine"),
        (PlayerInteractionKind::Shoe, "shoe"),
    ] {
        for state in 1u8..=2 {
            interaction_images.insert(
                (kind, state == 2),
                asset_server.load(format!("vendor/noname/interactions/{name}{state}.png")),
            );
            interaction_sounds.insert(
                (kind, state - 1),
                asset_server.load(format!(
                    "vendor/noname/interactions/throw_{name}{state}.mp3"
                )),
            );
        }
    }
    let interaction_cooldown_masks = (0..=INTERACTION_COOLDOWN_MASK_FRAMES)
        .map(|frame| {
            let fraction = frame as f32 / INTERACTION_COOLDOWN_MASK_FRAMES as f32;
            images.add(interaction_cooldown_mask_image(72, 62, fraction))
        })
        .collect();
    let deal_sounds = (1..=8)
        .map(|index| {
            asset_server.load(format!(
                "vendor/kenney/casino-audio/Audio/card-slide-{index}.ogg"
            ))
        })
        .collect();
    let place_sounds = (1..=4)
        .map(|index| {
            asset_server.load(format!(
                "vendor/kenney/casino-audio/Audio/card-place-{index}.ogg"
            ))
        })
        .collect();
    let shove_sounds = vec![asset_server.load("vendor/noname/audio/effect/flappybird_start.ogg")];
    let button_click_sounds = ["click-a.ogg", "click-b.ogg"]
        .map(|name| asset_server.load(format!("vendor/kenney/ui/Sounds/{name}")))
        .to_vec();
    let quick_voice_sounds = (0..QUICK_VOICE_COUNT)
        .map(|index| asset_server.load(format!("vendor/noname/voice/male/{index}.mp3")))
        .collect();
    commands.insert_resource(UiAssets {
        font: asset_server.load(UI_FONT_ASSET),
        playing_cards: PlayingCardAssets {
            cards,
            card_back: asset_server.load("vendor/kenney/boardgame/PNG/Cards/cardBack_blue4.png"),
        },
        controls: ControlAssets {
            primary_button: asset_server
                .load("vendor/kenney/ui/PNG/Green/Default/button_rectangle_depth_gradient.png"),
            secondary_button: asset_server
                .load("vendor/kenney/ui/PNG/Blue/Default/button_rectangle_depth_gradient.png"),
            warning_button: asset_server
                .load("vendor/kenney/ui/PNG/Yellow/Default/button_rectangle_depth_gradient.png"),
            danger_button: asset_server
                .load("vendor/kenney/ui/PNG/Red/Default/button_rectangle_depth_gradient.png"),
            disabled_button: asset_server
                .load("vendor/kenney/ui/PNG/Grey/Default/button_rectangle_depth_gradient.png"),
            robot_icon: asset_server.load("icons/robot-2-fill.png"),
            host_crown: asset_server.load("ui/home/cozy-host-crown.png"),
            github_mark: asset_server.load("icons/github-mark.png"),
        },
        home: HomeAssets {
            panel: asset_server.load("ui/home/cozy-panel.png"),
            warning_toast: asset_server.load("ui/home/cozy-toasts-atlas.png"),
            rule_left: asset_server.load("ui/home/rule-left.png"),
            rule_left_highlighted: asset_server.load("ui/home/rule-left-highlighted.png"),
            rule_right: asset_server.load("ui/home/rule-right.png"),
            rule_right_highlighted: asset_server.load("ui/home/rule-right-highlighted.png"),
            help_question: asset_server.load("ui/home/cozy-help-question.png"),
            checkbox: asset_server.load("ui/home/cozy-checkbox.png"),
            checkbox_highlighted: asset_server.load("ui/home/cozy-checkbox-highlighted.png"),
            checkbox_selected: asset_server.load("ui/home/cozy-checkbox-selected.png"),
            checkbox_selected_highlighted: asset_server
                .load("ui/home/cozy-checkbox-selected-highlighted.png"),
            button: asset_server.load("ui/home/cozy-button-compact.png"),
            purple_button: asset_server.load("ui/home/cozy-button-purple-plain.png"),
            purple_button_compact: asset_server.load("ui/home/cozy-button-purple-compact.png"),
            button_arrows: asset_server.load("ui/home/cozy-button-arrows.png"),
            close_button: asset_server.load("ui/home/cross_button.png"),
            close_button_highlighted: asset_server.load("ui/home/cross_button_highlighted.png"),
            slider: asset_server.load("ui/home/slider.png"),
            slider_highlighted: asset_server.load("ui/home/slider_highlighted.png"),
            slider_handle: asset_server.load("ui/home/slider_handle.png"),
            slider_handle_highlighted: asset_server.load("ui/home/slider_handle_highlighted.png"),
            input: asset_server.load("ui/home/cozy-input-compact.png"),
            focused_input: asset_server.load("ui/home/cozy-input-focused-compact.png"),
            game_art: [
                asset_server.load("vendor/kenney/boardgame/PNG/Cards/cardClubs7.png"),
                asset_server.load("vendor/kenney/boardgame/PNG/Cards/cardSpadesA.png"),
                asset_server.load("vendor/kenney/boardgame/PNG/Cards/cardHearts2.png"),
                asset_server.load("cards/uno/red_5.png"),
                asset_server.load("cards/mahjong/hong-kong/dragon-red.png"),
            ],
            // 与 profile::rating::REFERENCE_LEVEL_NAMES 的顺序一致。
            reference_level_icons: [
                asset_server.load("ui/profile/netherite.png"),
                asset_server.load("ui/profile/diamond.png"),
                asset_server.load("ui/profile/gold.png"),
                asset_server.load("ui/profile/redstone.png"),
                asset_server.load("ui/profile/iron.png"),
                asset_server.load("ui/profile/copper.png"),
                asset_server.load("ui/profile/cobblestone.png"),
                asset_server.load("ui/profile/oak_log.png"),
                asset_server.load("ui/profile/dirt.png"),
                asset_server.load("ui/profile/composter.png"),
            ],
        },
        social: SocialAssets {
            interaction_images,
            interaction_cooldown_masks,
            chat_emojis: CHAT_EMOJI_ASSET_PATHS
                .iter()
                .map(|path| asset_server.load(*path))
                .collect(),
            chat_emoji_icon: asset_server.load("icons/chat-emoji-white.png"),
            quick_voice_icon: asset_server.load("icons/list-menu.png"),
        },
        audio: CommonAudioAssets {
            interaction_sounds,
            deal_sounds,
            place_sounds,
            shove_sounds,
            button_click_sounds,
            error_popup_sound: asset_server
                .load("vendor/kenney/interface-sounds/Audio/error_007.ogg"),
            bomb_explosion_sound: asset_server.load("vendor/noname/damage_fire2.mp3"),
            summary_score_sound: asset_server
                .load("vendor/noname/audio/effect/flappybird_score.ogg"),
            summary_die_sound: asset_server.load("vendor/noname/audio/effect/flappybird_die.ogg"),
            quick_voice_sounds,
        },
        table_felt: asset_server.load(TABLE_FELT_ASSET),
    });
}
