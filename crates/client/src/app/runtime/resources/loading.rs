//! Bevy 场景基础节点与静态资源的统一加载。

use bevy::ui::FocusPolicy;
use leocard_protocol::{PlayerInteractionKind, QUICK_VOICE_COUNT};
use leocard_qigui523::build_deck;

use leocard_uno::{Mode, UnoRuleSet, build_deck_for_rules};
use std::collections::{HashMap, HashSet};

use super::*;

pub fn setup_camera(mut commands: Commands) {
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

pub fn load_ui_assets(
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
    let mut uno_cards = HashMap::new();
    for card in build_deck_for_rules(UnoRuleSet {
        swap_pack: true,
        reverse_pack: true,
        stack_pack: true,
        ..UnoRuleSet::default()
    }) {
        uno_cards
            .entry((card.color(), card.face()))
            .or_insert_with(|| asset_server.load::<Image>(uno_card_asset_path(card)));
    }
    for card in build_deck_for_rules(UnoRuleSet {
        mode: Mode::NoMercy,
        ..UnoRuleSet::default()
    }) {
        uno_cards
            .entry((card.color(), card.face()))
            .or_insert_with(|| asset_server.load::<Image>(uno_card_asset_path(card)));
    }
    for card in build_deck_for_rules(UnoRuleSet {
        mode: Mode::Flip,
        ..UnoRuleSet::default()
    }) {
        for face in [Some(card), card.opposite_public_face()] {
            let Some(face) = face else { continue };
            uno_cards
                .entry((face.color(), face.face()))
                .or_insert_with(|| asset_server.load::<Image>(uno_card_asset_path(face)));
        }
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
    let mahjong_kinds = leocard_mahjong::build_deck()
        .into_iter()
        .map(|tile| tile.kind())
        .collect::<HashSet<_>>();
    let mahjong_tiles = mahjong_kinds
        .iter()
        .copied()
        .map(|kind| (kind, asset_server.load(mahjong_tile_asset_path(kind))))
        .collect();
    let mahjong_tile_heights = mahjong_kinds
        .into_iter()
        .map(|kind| {
            (
                kind,
                asset_server.load(mahjong_tile_height_asset_path(kind)),
            )
        })
        .collect();

    commands.insert_resource(ShengjiSoundAssets::load(&asset_server));
    commands.insert_resource(UiAssets {
        font: asset_server.load(UI_FONT_ASSET),
        games: GameVisualAssets {
            cards,
            card_back: asset_server.load("vendor/kenney/boardgame/PNG/Cards/cardBack_blue4.png"),
            uno_cards,
            uno_card_back: asset_server.load("cards/uno/card_back.png"),
            mahjong_tiles,
            mahjong_tile_heights,
            mahjong_tile_back: asset_server.load("cards/mahjong/hong-kong/back.png"),
            mahjong_turn_arrow: asset_server
                .load("vendor/kenney/ui/PNG/Yellow/Default/arrow_basic_e.png"),
            poker_chips: HashMap::from([
                (
                    1,
                    asset_server.load("vendor/kenney/boardgame/PNG/Chips/chipWhiteBlue.png"),
                ),
                (
                    5,
                    asset_server.load("vendor/kenney/boardgame/PNG/Chips/chipRedWhite.png"),
                ),
                (
                    10,
                    asset_server.load("vendor/kenney/boardgame/PNG/Chips/chipBlueWhite.png"),
                ),
                (
                    25,
                    asset_server.load("vendor/kenney/boardgame/PNG/Chips/chipGreenWhite.png"),
                ),
                (
                    100,
                    asset_server.load("vendor/kenney/boardgame/PNG/Chips/chipBlackWhite.png"),
                ),
            ]),
            texas_sounds: TexasSoundAssets::load(&asset_server),
            uno_sounds: UnoSoundAssets::load(&asset_server),
            sequence_airplane: asset_server.load("ui/effects/sequence_airplane.png"),
            shengji_target: asset_server.load("ui/effects/shengji_target.png"),
            shengji_dart: asset_server.load("ui/effects/shengji_dart.png"),
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
            panel_window: asset_server.load("ui/panel_window.png"),
            panel_section: asset_server.load("ui/panel_section.png"),
            panel_popup: asset_server.load("ui/panel_popup.png"),
            player_panel_wide: asset_server.load("ui/player_panel_wide.png"),
            player_panel_compact: asset_server.load("ui/player_panel_compact.png"),
            robot_icon: asset_server.load("icons/robot-2-fill.png"),
            host_crown: asset_server.load("icons/host-crown.png"),
            github_mark: asset_server.load("icons/github-mark.png"),
        },
        social: SocialAssets {
            interaction_images,
            interaction_cooldown_masks,
            chat_emojis: CHAT_EMOJI_ASSET_PATHS
                .iter()
                .map(|path| asset_server.load(*path))
                .collect(),
            chat_emoji_icon: asset_server.load("icons/chat-emoji-white.png"),
            chat_open_icon: asset_server
                .load("vendor/kenney/ui/PNG/Blue/Default/arrow_basic_w.png"),
            chat_close_icon: asset_server
                .load("vendor/kenney/ui/PNG/Blue/Default/arrow_basic_e.png"),
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
