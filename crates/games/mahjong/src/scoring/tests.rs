use super::*;
use crate::{MahjongDragon, MahjongPlayerId, MahjongSuit, MahjongTileKind, MahjongWind, Meld};

fn c(suit: MahjongSuit, rank: u8) -> MahjongTileKind {
    MahjongTileKind::suited(suit, rank)
}

fn context(source: WinSource) -> WinContext {
    WinContext {
        source,
        seat_wind: MahjongWind::East,
        prevalent_wind: MahjongWind::East,
        last_wall_tile: false,
        last_of_kind: false,
        flower_count: 0,
    }
}

fn fan(result: &MahjongScoreResult, target: Fan) -> Option<FanValue> {
    result
        .fans
        .iter()
        .find(|value| value.fan == target)
        .copied()
}

#[test]
fn recognizes_thirteen_orphans_and_excludes_flowers_from_minimum() {
    let mut concealed = vec![
        c(MahjongSuit::Characters, 1),
        c(MahjongSuit::Characters, 9),
        c(MahjongSuit::Bamboo, 1),
        c(MahjongSuit::Bamboo, 9),
        c(MahjongSuit::Dots, 1),
        c(MahjongSuit::Dots, 9),
        MahjongTileKind::Wind(MahjongWind::East),
        MahjongTileKind::Wind(MahjongWind::South),
        MahjongTileKind::Wind(MahjongWind::West),
        MahjongTileKind::Wind(MahjongWind::North),
        MahjongTileKind::Dragon(MahjongDragon::Red),
        MahjongTileKind::Dragon(MahjongDragon::Green),
        MahjongTileKind::Dragon(MahjongDragon::White),
    ];
    concealed.push(MahjongTileKind::Wind(MahjongWind::East));
    let result = score_hand(&ScoreInput {
        winning_tile: MahjongTileKind::Wind(MahjongWind::East),
        concealed,
        melds: Vec::new(),
        context: WinContext {
            flower_count: 8,
            ..context(WinSource::SelfDraw)
        },
    })
    .unwrap();
    assert!(fan(&result, Fan::ThirteenOrphans).is_some());
    assert_eq!(result.points_without_flowers, 92);
    assert_eq!(result.total_points, 100);
}

#[test]
fn detects_pure_straight_and_does_not_repeat_short_straight() {
    let concealed = vec![
        c(MahjongSuit::Characters, 1),
        c(MahjongSuit::Characters, 2),
        c(MahjongSuit::Characters, 3),
        c(MahjongSuit::Characters, 4),
        c(MahjongSuit::Characters, 5),
        c(MahjongSuit::Characters, 6),
        c(MahjongSuit::Characters, 7),
        c(MahjongSuit::Characters, 8),
        c(MahjongSuit::Characters, 9),
        c(MahjongSuit::Dots, 2),
        c(MahjongSuit::Dots, 3),
        c(MahjongSuit::Dots, 4),
        MahjongTileKind::Dragon(MahjongDragon::Red),
        MahjongTileKind::Dragon(MahjongDragon::Red),
    ];
    let result = score_hand(&ScoreInput {
        concealed,
        melds: Vec::new(),
        winning_tile: MahjongTileKind::Dragon(MahjongDragon::Red),
        context: context(WinSource::Discard(MahjongPlayerId(1))),
    })
    .unwrap();
    assert!(fan(&result, Fan::PureStraight).is_some());
    assert!(fan(&result, Fan::ShortStraight).is_none());
    assert!(fan(&result, Fan::TwoTerminalChows).is_none());
}

#[test]
fn mixed_melded_and_concealed_kong_scores_six_in_2014_rules() {
    let concealed = vec![
        c(MahjongSuit::Characters, 3),
        c(MahjongSuit::Characters, 4),
        c(MahjongSuit::Characters, 5),
        c(MahjongSuit::Dots, 7),
        c(MahjongSuit::Dots, 7),
    ];
    let melds = vec![
        Meld::melded_kong(c(MahjongSuit::Bamboo, 2), MahjongPlayerId(1)),
        Meld::concealed_kong(c(MahjongSuit::Dots, 4)),
        Meld::pung(
            MahjongTileKind::Dragon(MahjongDragon::Red),
            MahjongPlayerId(2),
        ),
    ];
    let result = score_hand(&ScoreInput {
        concealed,
        melds,
        winning_tile: c(MahjongSuit::Dots, 7),
        context: context(WinSource::Discard(MahjongPlayerId(3))),
    })
    .unwrap();
    assert_eq!(fan(&result, Fan::TwoMeldedKongs).unwrap().points, 6);
    assert!(fan(&result, Fan::MeldedKong).is_none());
    assert!(fan(&result, Fan::ConcealedKong).is_none());
}

#[test]
fn knitted_straight_keeps_the_remaining_set_and_pair_fans() {
    let concealed = vec![
        c(MahjongSuit::Characters, 1),
        c(MahjongSuit::Characters, 4),
        c(MahjongSuit::Characters, 7),
        c(MahjongSuit::Bamboo, 2),
        c(MahjongSuit::Bamboo, 5),
        c(MahjongSuit::Bamboo, 8),
        c(MahjongSuit::Dots, 3),
        c(MahjongSuit::Dots, 6),
        c(MahjongSuit::Dots, 9),
        MahjongTileKind::Dragon(MahjongDragon::Red),
        MahjongTileKind::Dragon(MahjongDragon::Red),
        MahjongTileKind::Dragon(MahjongDragon::Red),
        MahjongTileKind::Wind(MahjongWind::East),
        MahjongTileKind::Wind(MahjongWind::East),
    ];
    let result = score_hand(&ScoreInput {
        concealed,
        melds: Vec::new(),
        winning_tile: MahjongTileKind::Wind(MahjongWind::East),
        context: context(WinSource::Discard(MahjongPlayerId(1))),
    })
    .unwrap();
    assert!(fan(&result, Fan::KnittedStraight).is_some());
    assert!(fan(&result, Fan::AllTypes).is_some());
    assert!(fan(&result, Fan::DragonPung).is_some());
    assert!(fan(&result, Fan::ConcealedHand).is_some());
    assert!(fan(&result, Fan::SingleWait).is_some());
}
