use super::*;
use crate::{
    MahjongFan, MahjongMatchLength, MahjongMeld, MahjongRuleSet, MahjongSuit, MahjongTile,
    MahjongTileKind, MahjongWinSource, MahjongWind, build_deck,
};
use leocard_game_common::PlayerId;
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
        MahjongGameState::new_with_deck(MahjongRuleSet::default(), build_deck(), PlayerId(0))
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
    game.phase = MahjongPhase::Playing;
    assert!(matches!(
        game.self_draw_win_availability(PlayerId(0)).unwrap(),
        MahjongWinAvailability::InsufficientFan { points: 1..=7 }
    ));
    let winning = game.players[0].hand.pop().unwrap();
    assert!(matches!(
        game.claim_win_availability(
            PlayerId(0),
            winning.kind(),
            MahjongWinSource::Discard(PlayerId(1))
        )
        .unwrap(),
        MahjongWinAvailability::InsufficientFan { .. }
    ));
    game.rules.minimum_eight_points = false;
    assert_eq!(
        game.claim_win_availability(
            PlayerId(0),
            winning.kind(),
            MahjongWinSource::Discard(PlayerId(1))
        )
        .unwrap(),
        MahjongWinAvailability::Legal
    );
    game.players[0].dead_hand = true;
    assert_eq!(
        game.claim_win_availability(
            PlayerId(0),
            winning.kind(),
            MahjongWinSource::Discard(PlayerId(1))
        )
        .unwrap(),
        MahjongWinAvailability::Unavailable
    );
}

#[test]
fn settlement_records_nine_gates_waits_before_adding_the_winning_tile() {
    let mut game =
        MahjongGameState::new_with_deck(MahjongRuleSet::default(), build_deck(), PlayerId(0))
            .unwrap();
    game.players[0].hand = tiles(
        [1, 1, 1, 2, 3, 4, 5, 6, 7, 8, 9, 9, 9, 5]
            .map(|rank| MahjongTileKind::suited(MahjongSuit::Characters, rank)),
    );
    game.last_drawn = game.players[0].hand.last().copied();
    game.phase = MahjongPhase::Playing;
    let MahjongActionOutcome::HandFinished(result) = game.declare_self_draw(PlayerId(0)).unwrap()
    else {
        panic!("must win");
    };
    assert_eq!(result.winners[0].wait_kind_count, 9);
    assert!(result.winners[0].concealed_hand);
    assert_eq!(result.match_length, MahjongMatchLength::SingleHand);
    assert_eq!(result.match_progress.completed_hands, 1);
    assert_eq!(result.match_progress.exhaustive_draws, 0);
}

#[test]
fn balanced_self_draw_scores_seven_pairs_and_uses_the_configured_base() {
    for (balanced_scoring, minimum_eight_points, seven_pairs, base) in [
        (false, true, 24, 8),
        (true, true, 16, 4),
        (true, false, 16, 0),
    ] {
        let rules = MahjongRuleSet {
            balanced_scoring,
            minimum_eight_points,
            ..MahjongRuleSet::default()
        };
        let player = PlayerId(0);
        let mut game = MahjongGameState::new_with_deck(rules, build_deck(), player).unwrap();
        let pairs = [
            MahjongTileKind::suited(MahjongSuit::Characters, 1),
            MahjongTileKind::suited(MahjongSuit::Characters, 2),
            MahjongTileKind::suited(MahjongSuit::Dots, 3),
            MahjongTileKind::suited(MahjongSuit::Dots, 4),
            MahjongTileKind::suited(MahjongSuit::Bamboo, 5),
            MahjongTileKind::suited(MahjongSuit::Bamboo, 6),
            MahjongTileKind::Wind(MahjongWind::East),
        ];
        game.players[player.index()].hand = tiles(pairs.into_iter().flat_map(|tile| [tile; 2]));
        game.last_drawn = game.players[player.index()].hand.last().copied();
        game.phase = MahjongPhase::Playing;
        let MahjongActionOutcome::HandFinished(result) = game.declare_self_draw(player).unwrap()
        else {
            panic!("seven pairs must win");
        };
        let score = &result.winners[0].score;
        assert!(
            score
                .fans
                .iter()
                .any(|fan| { fan.fan == MahjongFan::SevenPairs && fan.points == seven_pairs })
        );
        let payment = i32::from(score.total_points) + base;
        assert_eq!(result.deltas, [payment * 3, -payment, -payment, -payment]);
    }
}

#[test]
fn concealed_shape_in_settlement_allows_concealed_kongs_but_excludes_open_melds() {
    let east = MahjongTileKind::Wind(MahjongWind::East);
    for (meld, concealed) in [
        (MahjongMeld::concealed_kong(east), true),
        (MahjongMeld::melded_kong(east, PlayerId(1)), false),
        (MahjongMeld::pung(east, PlayerId(1)), false),
    ] {
        let mut game =
            MahjongGameState::new_with_deck(MahjongRuleSet::default(), build_deck(), PlayerId(0))
                .unwrap();
        game.players[0].hand = tiles(
            [1, 1, 1, 2, 2, 2, 3, 3, 3, 5, 5]
                .map(|rank| MahjongTileKind::suited(MahjongSuit::Characters, rank)),
        );
        game.players[0].melds = vec![meld];
        game.last_drawn = game.players[0].hand.last().copied();
        game.phase = MahjongPhase::Playing;
        let MahjongActionOutcome::HandFinished(result) =
            game.declare_self_draw(PlayerId(0)).unwrap()
        else {
            panic!("must win");
        };
        assert_eq!(result.winners[0].concealed_hand, concealed);
    }
}

#[test]
fn match_draw_totals_span_hands_and_reset_for_the_next_match() {
    let mut game = MahjongGameState::new_with_deck(
        MahjongRuleSet {
            match_length: MahjongMatchLength::EastRound,
            ..MahjongRuleSet::default()
        },
        build_deck(),
        PlayerId(0),
    )
    .unwrap();
    for index in 1..=4 {
        let MahjongActionOutcome::HandFinished(result) = game.finish_exhaustive_draw().unwrap()
        else {
            panic!("must finish");
        };
        assert_eq!(result.match_progress.completed_hands, index);
        assert_eq!(result.match_progress.exhaustive_draws, index);
        assert_eq!(result.match_complete, index == 4);
        game.start_next_hand(build_deck()).unwrap();
    }
    assert_eq!(game.match_progress, MahjongMatchProgress::default());
}
