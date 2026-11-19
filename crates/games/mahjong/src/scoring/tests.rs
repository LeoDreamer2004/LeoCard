use crate::{
    MahjongFan, MahjongMeld, MahjongScoreInput, MahjongSuit, MahjongTileKind, MahjongWinContext,
    MahjongWinSource, MahjongWind, score_hand, waiting_tile_kinds,
};
use leocard_game_common::PlayerId;

fn chow(suit: MahjongSuit, start: u8) -> [MahjongTileKind; 3] {
    [start, start + 1, start + 2].map(|rank| MahjongTileKind::suited(suit, rank))
}

fn input(
    concealed: Vec<MahjongTileKind>,
    melds: Vec<MahjongMeld>,
    winning_tile: MahjongTileKind,
) -> MahjongScoreInput {
    MahjongScoreInput {
        concealed,
        melds,
        winning_tile,
        context: MahjongWinContext {
            declared_ready: false,
            source: MahjongWinSource::Discard(PlayerId(1)),
            seat_wind: MahjongWind::South,
            prevalent_wind: MahjongWind::West,
            last_wall_tile: false,
            last_of_kind: false,
            flower_count: 0,
        },
    }
}

#[test]
fn melded_chows_do_not_turn_a_pair_wait_into_an_edge_or_closed_wait() {
    for (start, rank) in [(2, 3), (1, 3), (7, 7)] {
        let winning = MahjongTileKind::suited(MahjongSuit::Dots, rank);
        let sets = [
            (MahjongSuit::Dots, start),
            (MahjongSuit::Characters, 1),
            (MahjongSuit::Characters, 4),
            (MahjongSuit::Bamboo, 7),
        ];
        for open_count in 1..=4 {
            let mut concealed = vec![winning; 2];
            let mut melds = Vec::new();
            for (index, (suit, start)) in sets.into_iter().enumerate() {
                if index < open_count {
                    melds.push(MahjongMeld::chow(suit, start, PlayerId(1)));
                } else {
                    concealed.extend(chow(suit, start));
                }
            }
            let result = score_hand(&input(concealed, melds, winning)).unwrap();
            assert!(
                result.fans.iter().all(|value| !matches!(
                    value.fan,
                    MahjongFan::EdgeWait | MahjongFan::ClosedWait
                )),
                "open_count={open_count}, winning={winning:?}, fans={:?}",
                result.fans
            );
            assert_eq!(
                result
                    .fans
                    .iter()
                    .any(|value| value.fan == MahjongFan::SingleWait),
                open_count < 4,
            );
            assert_eq!(
                result
                    .fans
                    .iter()
                    .any(|value| value.fan == MahjongFan::MeldedHand),
                open_count == 4,
            );
        }
    }
    let winning = MahjongTileKind::suited(MahjongSuit::Dots, 3);
    let result = score_hand(&input(
        vec![winning; 2],
        vec![
            MahjongMeld::chow(MahjongSuit::Dots, 2, PlayerId(1)),
            MahjongMeld::chow(MahjongSuit::Characters, 4, PlayerId(1)),
            MahjongMeld::chow(MahjongSuit::Bamboo, 7, PlayerId(1)),
            MahjongMeld::pung(MahjongTileKind::Wind(MahjongWind::East), PlayerId(1)),
        ],
        winning,
    ))
    .unwrap();
    // Melded Hand and a terminal/honor pung total seven, below the eight-point minimum.
    assert_eq!(result.points_without_flowers, 7);
}

#[test]
fn concealed_chow_waits_still_score_after_calling_another_chow() {
    for (start, rank, expected) in [
        (2, 3, MahjongFan::ClosedWait),
        (1, 3, MahjongFan::EdgeWait),
        (7, 7, MahjongFan::EdgeWait),
    ] {
        let mut concealed = chow(MahjongSuit::Dots, start).to_vec();
        concealed.extend(chow(MahjongSuit::Characters, 4));
        concealed.extend(chow(MahjongSuit::Bamboo, 7));
        concealed.extend([MahjongTileKind::Wind(MahjongWind::East); 2]);
        let result = score_hand(&input(
            concealed,
            vec![MahjongMeld::chow(MahjongSuit::Characters, 1, PlayerId(1))],
            MahjongTileKind::suited(MahjongSuit::Dots, rank),
        ))
        .unwrap();
        let waits: Vec<_> = result
            .fans
            .iter()
            .filter(|value| {
                matches!(
                    value.fan,
                    MahjongFan::EdgeWait | MahjongFan::ClosedWait | MahjongFan::SingleWait
                )
            })
            .map(|value| value.fan)
            .collect();
        assert_eq!(waits, vec![expected]);
    }
}

#[test]
fn a_pung_wait_is_not_reclassified_by_a_melded_chow() {
    let pair = MahjongTileKind::suited(MahjongSuit::Dots, 6);
    let winning = MahjongTileKind::suited(MahjongSuit::Dots, 7);
    let mut concealed = vec![MahjongTileKind::Wind(MahjongWind::East); 3];
    concealed.extend([pair; 2]);
    concealed.extend([winning; 2]);
    let melds = vec![
        MahjongMeld::chow(MahjongSuit::Dots, 6, PlayerId(1)),
        MahjongMeld::chow(MahjongSuit::Dots, 4, PlayerId(1)),
    ];
    // All four sixes are already held, so only the seven can complete this hand.
    assert_eq!(waiting_tile_kinds(&concealed, &melds), vec![winning]);
    concealed.push(winning);
    let result = score_hand(&input(concealed, melds, winning)).unwrap();
    assert!(result.fans.iter().all(|value| !matches!(
        value.fan,
        MahjongFan::EdgeWait | MahjongFan::ClosedWait | MahjongFan::SingleWait
    )));
    assert!(
        result
            .fans
            .iter()
            .all(|value| value.fan != MahjongFan::TwoConcealedPungs)
    );
}
