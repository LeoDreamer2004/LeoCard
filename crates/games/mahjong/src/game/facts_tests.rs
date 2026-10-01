use super::*;
use crate::{
    MahjongMatchLength, MahjongPlayerId, MahjongRuleSet, MahjongSuit, MahjongTile, MahjongTileKind,
    MahjongWind, WinSource, build_deck,
};
use std::collections::BTreeMap;

fn tiles(kinds: impl IntoIterator<Item = MahjongTileKind>) -> Vec<MahjongTile> {
    let mut copies = BTreeMap::new();
    kinds
        .into_iter()
        .map(|kind| {
            let copy = copies.entry(kind).or_insert(0);
            let tile = MahjongTile::new(kind, *copy);
            *copy += 1;
            tile
        })
        .collect()
}

#[test]
fn win_availability_distinguishes_incomplete_shape_from_insufficient_fan() {
    let mut game =
        GameState::new_with_deck(MahjongRuleSet::default(), build_deck(), MahjongPlayerId(0))
            .unwrap();
    let mut hand = Vec::new();
    for (suit, ranks) in [
        (MahjongSuit::Characters, vec![1, 2, 3, 4, 5, 6]),
        (MahjongSuit::Bamboo, vec![2, 3, 4]),
        (MahjongSuit::Dots, vec![5, 6, 7]),
    ] {
        hand.extend(
            ranks
                .into_iter()
                .map(|rank| MahjongTileKind::suited(suit, rank)),
        );
    }
    hand.extend([MahjongTileKind::Wind(MahjongWind::East); 2]);
    game.players[0].hand = tiles(hand);
    game.last_drawn = game.players[0].hand.last().copied();
    game.phase = Phase::Playing;
    assert!(matches!(
        game.self_draw_win_availability(MahjongPlayerId(0)).unwrap(),
        MahjongWinAvailability::InsufficientFan { points: 1..=7 }
    ));
    let winning = game.players[0].hand.pop().unwrap();
    assert!(matches!(
        game.claim_win_availability(
            MahjongPlayerId(0),
            winning.kind(),
            WinSource::Discard(MahjongPlayerId(1))
        )
        .unwrap(),
        MahjongWinAvailability::InsufficientFan { .. }
    ));
    game.rules.minimum_eight_points = false;
    assert_eq!(
        game.claim_win_availability(
            MahjongPlayerId(0),
            winning.kind(),
            WinSource::Discard(MahjongPlayerId(1))
        )
        .unwrap(),
        MahjongWinAvailability::Legal
    );
    game.players[0].dead_hand = true;
    assert_eq!(
        game.claim_win_availability(
            MahjongPlayerId(0),
            winning.kind(),
            WinSource::Discard(MahjongPlayerId(1))
        )
        .unwrap(),
        MahjongWinAvailability::Unavailable
    );
}

#[test]
fn settlement_records_nine_gates_waits_before_adding_the_winning_tile() {
    let mut game =
        GameState::new_with_deck(MahjongRuleSet::default(), build_deck(), MahjongPlayerId(0))
            .unwrap();
    game.players[0].hand = tiles(
        [1, 1, 1, 2, 3, 4, 5, 6, 7, 8, 9, 9, 9, 5]
            .map(|rank| MahjongTileKind::suited(MahjongSuit::Characters, rank)),
    );
    game.last_drawn = game.players[0].hand.last().copied();
    game.phase = Phase::Playing;
    let ActionOutcome::HandFinished(result) = game.declare_self_draw(MahjongPlayerId(0)).unwrap()
    else {
        panic!("must win");
    };
    assert_eq!(result.winners[0].wait_kind_count, 9);
    assert_eq!(result.match_length, MahjongMatchLength::SingleHand);
    assert_eq!(result.match_progress.completed_hands, 1);
    assert_eq!(result.match_progress.exhaustive_draws, 0);
}

#[test]
fn match_draw_totals_span_hands_and_reset_for_the_next_match() {
    let mut game = GameState::new_with_deck(
        MahjongRuleSet {
            match_length: MahjongMatchLength::EastRound,
            ..MahjongRuleSet::default()
        },
        build_deck(),
        MahjongPlayerId(0),
    )
    .unwrap();
    for index in 1..=4 {
        let ActionOutcome::HandFinished(result) = game.finish_exhaustive_draw().unwrap() else {
            panic!("must finish");
        };
        assert_eq!(result.match_progress.completed_hands, index);
        assert_eq!(result.match_progress.exhaustive_draws, index);
        assert_eq!(result.match_complete, index == 4);
        game.start_next_hand(build_deck()).unwrap();
    }
    assert_eq!(game.match_progress, MahjongMatchProgress::default());
}
