use super::prelude::*;
use bevy::audio::Decodable;
use leocard_protocol::{
    AVATAR_DIMENSION, MAX_AVATAR_BYTES, MAX_CHAT_MESSAGE_CHARS, QUICK_VOICE_COUNT,
};
use leocard_qigui523::build_deck;
use leocard_uno::{Mode, UnoRuleSet, build_deck_for_rules};
use std::collections::HashSet;
use std::io::Cursor;

#[test]
fn every_card_maps_to_an_existing_asset() {
    let asset_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets");
    for card in build_deck(1) {
        let path = asset_root.join(card_asset_path(card.rank(), card.suit()));
        assert!(path.is_file(), "missing card asset: {}", path.display());
    }
    let mahjong_kinds = leocard_mahjong::build_deck()
        .into_iter()
        .map(|tile| tile.kind())
        .collect::<HashSet<_>>();
    assert_eq!(mahjong_kinds.len(), 42);
    for kind in mahjong_kinds {
        let path = asset_root.join(mahjong_tile_asset_path(kind));
        let image = image::open(&path)
            .unwrap_or_else(|error| panic!("{} 无法解码：{error}", path.display()));
        assert_eq!((image.width(), image.height()), (600, 800));
        let height_path = asset_root.join(mahjong_tile_height_asset_path(kind));
        let height = image::open(&height_path)
            .unwrap_or_else(|error| panic!("{} 无法解码：{error}", height_path.display()));
        assert_eq!((height.width(), height.height()), (600, 800));
    }
    let mahjong_back = asset_root.join("cards/mahjong/hong-kong/back.png");
    assert!(mahjong_back.is_file());
    for card in build_deck_for_rules(UnoRuleSet {
        mode: Mode::NoMercy,
        ..UnoRuleSet::default()
    }) {
        let path = asset_root.join(uno_card_asset_path(card));
        assert!(
            path.is_file(),
            "missing No Mercy card asset: {}",
            path.display()
        );
    }
    assert!(asset_root.join(TABLE_FELT_ASSET).is_file());
    assert!(asset_root.join(UI_FONT_ASSET).is_file());
    assert!(asset_root.join("fonts/OFL-ChillRoundGothic.txt").is_file());
    assert!(asset_root.join("icons/list-menu.png").is_file());
    assert!(asset_root.join("icons/github-mark.png").is_file());
    assert!(
        asset_root
            .join("ui/effects/sequence_airplane.png")
            .is_file()
    );
    assert!(
        asset_root
            .join("ui/effects/sequence_airplane.svg")
            .is_file()
    );
    assert!(asset_root.join("ui/effects/shengji_target.png").is_file());
    assert!(asset_root.join("ui/effects/shengji_dart.png").is_file());
}

#[test]
fn every_runtime_ui_and_card_sound_decodes_with_enabled_bevy_formats() {
    let asset_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets");
    let mut sounds = vec![
        "vendor/kenney/ui/Sounds/click-a.ogg".to_owned(),
        "vendor/kenney/ui/Sounds/click-b.ogg".to_owned(),
        "vendor/kenney/interface-sounds/Audio/error_007.ogg".to_owned(),
    ];
    for (prefix, count) in [
        ("card-slide", 8),
        ("card-place", 4),
        ("card-shove", 4),
        ("card-fan", 2),
        ("chip-lay", 3),
        ("chips-collide", 4),
        ("chips-handle", 6),
        ("chips-stack", 6),
    ] {
        sounds.extend(
            (1..=count)
                .map(|index| format!("vendor/kenney/casino-audio/Audio/{prefix}-{index}.ogg")),
        );
    }
    sounds.extend(["tap-a.ogg", "tap-b.ogg"].map(|name| format!("vendor/kenney/ui/Sounds/{name}")));
    sounds.extend(
        [
            "bong_001.ogg",
            "drop_004.ogg",
            "switch_003.ogg",
            "switch_004.ogg",
            "pluck_001.ogg",
            "pluck_002.ogg",
            "confirmation_001.ogg",
            "confirmation_002.ogg",
            "confirmation_003.ogg",
            "confirmation_004.ogg",
            "scratch_004.ogg",
        ]
        .map(|name| format!("vendor/kenney/interface-sounds/Audio/{name}")),
    );
    for (prefix, count) in [
        ("close", 4),
        ("error", 8),
        ("glass", 6),
        ("maximize", 9),
        ("question", 4),
        ("scroll", 5),
        ("select", 8),
    ] {
        sounds.extend(
            (1..=count).map(|index| {
                format!("vendor/kenney/interface-sounds/Audio/{prefix}_00{index}.ogg")
            }),
        );
    }
    sounds.push("vendor/noname/audio/effect/flappybird_start.ogg".to_owned());
    sounds.push("vendor/noname/audio/effect/flappybird_score.ogg".to_owned());
    sounds.push("vendor/noname/audio/effect/flappybird_die.ogg".to_owned());
    sounds.push("audio/shengji/power-off.ogg".to_owned());
    sounds.push("audio/shengji/power-on.ogg".to_owned());
    sounds.extend(
        (0..QUICK_VOICE_COUNT).map(|index| format!("vendor/noname/voice/male/{index}.mp3")),
    );
    sounds.push("vendor/noname/damage_fire2.mp3".to_owned());

    for relative in sounds {
        let path = asset_root.join(&relative);
        let source = AudioSource {
            bytes: std::fs::read(&path).unwrap().into(),
        };
        let mut decoder = source.decoder();
        assert!(
            decoder.next().is_some(),
            "audio contains no decodable samples: {}",
            path.display()
        );
    }
}

#[test]
fn chat_input_obeys_the_protocol_character_limit() {
    let mut input = "你好".to_owned();
    append_chat_input(&mut input, &"界".repeat(MAX_CHAT_MESSAGE_CHARS));
    assert_eq!(input.chars().count(), MAX_CHAT_MESSAGE_CHARS);
    assert_eq!(QUICK_VOICES.len(), usize::from(QUICK_VOICE_COUNT));
}

#[test]
fn avatar_is_cropped_and_encoded_as_bounded_64px_png() {
    let source = image::DynamicImage::ImageRgba8(image::ImageBuffer::from_pixel(
        80,
        40,
        image::Rgba([210, 80, 30, 255]),
    ));
    let mut encoded = Cursor::new(Vec::new());
    source
        .write_to(&mut encoded, image::ImageFormat::Png)
        .unwrap();

    let normalized = normalize_avatar_bytes(&encoded.into_inner()).unwrap();
    let decoded =
        image::load_from_memory_with_format(&normalized, image::ImageFormat::Png).unwrap();

    assert_eq!(decoded.width(), AVATAR_DIMENSION);
    assert_eq!(decoded.height(), AVATAR_DIMENSION);
    assert!(valid_normalized_avatar(&normalized));
    assert!(normalized.len() <= MAX_AVATAR_BYTES);
}

#[test]
fn jpeg_avatar_is_accepted_and_normalized_to_png() {
    let source = image::DynamicImage::ImageRgb8(image::ImageBuffer::from_pixel(
        48,
        80,
        image::Rgb([35, 140, 210]),
    ));
    let mut encoded = Cursor::new(Vec::new());
    source
        .write_to(&mut encoded, image::ImageFormat::Jpeg)
        .unwrap();

    let normalized = normalize_avatar_bytes(&encoded.into_inner()).unwrap();
    let decoded =
        image::load_from_memory_with_format(&normalized, image::ImageFormat::Png).unwrap();

    assert!(normalized.starts_with(b"\x89PNG\r\n\x1a\n"));
    assert_eq!(decoded.width(), AVATAR_DIMENSION);
    assert_eq!(decoded.height(), AVATAR_DIMENSION);
    assert!(valid_normalized_avatar(&normalized));
    assert!(normalized.len() <= MAX_AVATAR_BYTES);
}
