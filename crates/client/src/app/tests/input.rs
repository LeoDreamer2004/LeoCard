use super::*;
#[cfg(feature = "developer")]
use leocard_mahjong::{MahjongSuit, MahjongTileKind, MahjongWind};
use leocard_protocol::{GameViolation, RejectReason, RuleViolation};
#[cfg(feature = "developer")]
use leocard_qigui523::{QiGuiCard, QiGuiRank, QiGuiSuit};
#[cfg(feature = "developer")]
use std::collections::HashSet;

#[test]
fn not_players_turn_rejection_does_not_create_a_popup() {
    assert_eq!(
        rejection_label(&RejectReason::Game(GameViolation::QiGui523(
            RuleViolation::NotPlayersTurn,
        ))),
        None
    );
    assert!(
        rejection_label(&RejectReason::Game(GameViolation::QiGui523(
            RuleViolation::InvalidPattern,
        )))
        .is_some()
    );
}

#[test]
fn drag_selection_range_works_in_both_directions() {
    let mut drag = CardDragSelection {
        active: true,
        anchor: 5,
        current: 2,
        select: true,
    };
    assert!(!drag.contains(1));
    assert!(drag.contains(2));
    assert!(drag.contains(4));
    assert!(drag.contains(5));
    assert!(!drag.contains(6));

    drag.active = false;
    assert!(!drag.contains(4));
}

#[test]
fn ui_scale_fits_design_size_and_respects_manual_zoom() {
    assert_eq!(calculate_ui_scale(1280.0, 720.0, 1.0), 1.0);
    assert_eq!(calculate_ui_scale(640.0, 400.0, 1.0), 0.5);
    assert_eq!(calculate_ui_scale(2560.0, 1440.0, 1.0), 2.0);
    assert_eq!(calculate_ui_scale(1280.0, 720.0, 1.5), 1.5);
    assert_eq!(calculate_ui_scale(1280.0, 720.0, 10.0), 1.5);
}

#[test]
fn table_brightness_is_normalized_and_mapped_to_the_slider() {
    assert_eq!(MIN_TABLE_BRIGHTNESS, 0.1);
    assert_eq!(normalize_table_brightness(0.0), 1.0);
    assert_eq!(normalize_table_brightness(f32::NAN), 1.0);
    assert_eq!(table_brightness_fraction(MIN_TABLE_BRIGHTNESS), 0.0);
    assert_eq!(table_brightness_fraction(MAX_TABLE_BRIGHTNESS), 1.0);
    assert_eq!(slider_fraction_from_relative_x(-0.5), 0.0);
    assert_eq!(slider_fraction_from_relative_x(0.0), 0.5);
    assert_eq!(slider_fraction_from_relative_x(0.5), 1.0);
    assert_eq!(slider_fraction_from_relative_x(-2.0), 0.0);
    assert_eq!(slider_fraction_from_relative_x(2.0), 1.0);
}

#[test]
fn table_material_parameters_clamp_invalid_visual_settings() {
    let params = table_material_params(f32::NAN, 5.0, true);
    assert_eq!(params.x, DEFAULT_TABLE_VIGNETTE);
    assert_eq!(params.y, 1.0);
    assert_eq!(params.z, 1.0);
    assert_eq!(params.w, 0.0);

    let custom = table_material_params(1.0, DEFAULT_TABLE_VIGNETTE, false);
    assert_eq!(custom.z, 0.0);
}

#[test]
fn table_background_shader_tiles_builtin_felt_and_cover_crops_custom_images() {
    let shader = include_str!("../../../../../assets/shaders/table_background.wgsl");
    assert!(shader.contains("let cover_scale = max("));
    assert!(shader.contains("let crop_origin ="));
    assert!(shader.contains("return fract("));
}

#[cfg(feature = "developer")]
#[test]
fn developer_hand_parser_accepts_compact_cards_and_assigns_physical_copies() {
    let cards = parse_developer_hand("s4 H5,c6;dK ST").unwrap();
    assert_eq!(
        cards,
        vec![
            QiGuiCard::suited(0, QiGuiSuit::Spade, QiGuiRank::Four),
            QiGuiCard::suited(0, QiGuiSuit::Heart, QiGuiRank::Five),
            QiGuiCard::suited(0, QiGuiSuit::Club, QiGuiRank::Six),
            QiGuiCard::suited(0, QiGuiSuit::Diamond, QiGuiRank::King),
            QiGuiCard::suited(0, QiGuiSuit::Spade, QiGuiRank::Ten),
        ]
    );

    let copies = parse_developer_hand("S4S4S4").unwrap();
    assert_eq!(copies[0].deck(), 0);
    assert_eq!(copies[1].deck(), 1);
    assert_eq!(copies[2].deck(), 2);
    assert!(parse_developer_hand("S10").is_err());
}

#[cfg(feature = "developer")]
#[test]
fn developer_hand_parser_supports_red_and_black_jokers() {
    assert_eq!(
        parse_developer_hand("RJBJ").unwrap(),
        vec![
            QiGuiCard::suited(0, QiGuiSuit::Spade, QiGuiRank::Joker),
            QiGuiCard::suited(0, QiGuiSuit::Club, QiGuiRank::Joker),
        ]
    );
    assert!(parse_developer_hand("S0C0").is_err());
    assert_eq!(
        parse_developer_hand("SJCJ").unwrap(),
        vec![
            QiGuiCard::suited(0, QiGuiSuit::Spade, QiGuiRank::Jack),
            QiGuiCard::suited(0, QiGuiSuit::Club, QiGuiRank::Jack),
        ]
    );
    assert!(parse_developer_hand("SJOKER").is_err());
    let small_jokers = parse_developer_hand("BJBJ").unwrap();
    assert_eq!(
        small_jokers[0],
        QiGuiCard::suited(0, QiGuiSuit::Club, QiGuiRank::Joker)
    );
    assert_eq!(
        small_jokers[1],
        QiGuiCard::suited(1, QiGuiSuit::Club, QiGuiRank::Joker)
    );
}

#[cfg(feature = "developer")]
#[test]
fn developer_hand_parser_randomizes_suits_in_rank_only_mode() {
    let cards = parse_developer_hand("70523").unwrap();
    assert_eq!(
        cards
            .iter()
            .copied()
            .map(QiGuiCard::rank)
            .collect::<Vec<_>>(),
        vec![
            QiGuiRank::Seven,
            QiGuiRank::Joker,
            QiGuiRank::Five,
            QiGuiRank::Two,
            QiGuiRank::Three,
        ]
    );
    assert!(matches!(
        cards[1].suit(),
        QiGuiSuit::Spade | QiGuiSuit::Club
    ));
    assert_eq!(cards.iter().copied().collect::<HashSet<_>>().len(), 5);
    let repeated = parse_developer_hand("77777000").unwrap();
    assert_eq!(repeated.len(), 8);
    assert_eq!(
        repeated.iter().copied().collect::<HashSet<_>>().len(),
        repeated.len()
    );
}

#[cfg(feature = "developer")]
#[test]
fn developer_mahjong_hand_parser_supports_suits_and_honors() {
    let tiles = parse_developer_mahjong_hand("123m 456p 789s 1234z").unwrap();
    assert_eq!(tiles.len(), 13);
    assert_eq!(
        &tiles[..3],
        &[
            MahjongTileKind::suited(MahjongSuit::Characters, 1),
            MahjongTileKind::suited(MahjongSuit::Characters, 2),
            MahjongTileKind::suited(MahjongSuit::Characters, 3),
        ]
    );
    assert_eq!(tiles[9], MahjongTileKind::Wind(MahjongWind::East));
    assert_eq!(tiles[12], MahjongTileKind::Wind(MahjongWind::North));
    assert!(parse_developer_mahjong_hand("8Z").is_err());
    assert!(parse_developer_mahjong_hand("123M4").is_err());
}
