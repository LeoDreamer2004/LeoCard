use super::cards;
use crate::{
    DoudizhuAction, DoudizhuGameError, DoudizhuGameState, DoudizhuMatchState, DoudizhuMode,
    DoudizhuResultView, DoudizhuRuleSet, build_deck,
};
use leocard_game_common::PlayerId;

fn result(scores: Vec<i32>) -> DoudizhuResultView {
    DoudizhuResultView {
        winner: PlayerId(0),
        landlord_won: true,
        spring: false,
        scores,
    }
}

#[test]
fn every_round_rotates_first_bidder_and_counts_each_hand_once() {
    for mode in [DoudizhuMode::Three, DoudizhuMode::Four] {
        let mut state = DoudizhuMatchState::new(
            DoudizhuRuleSet {
                mode,
                rounds: 5,
                ..DoudizhuRuleSet::default()
            },
            PlayerId(1),
        )
        .unwrap();
        assert!(!state.next_hand());
        for hand in 1..=5 * mode.players() {
            assert_eq!(state.progress().hand_number, hand);
            assert_eq!(state.first_bidder(), PlayerId(hand % mode.players()));
            assert_eq!(state.progress().round(), (hand - 1) / mode.players() + 1);
            assert_eq!(
                state.progress().hand_in_round(),
                (hand - 1) % mode.players() + 1
            );
            assert!(state.reference_deltas(mode).is_none());
            let mut scores = vec![-1; usize::from(mode.players())];
            scores[0] = i32::from(mode.players() - 1);
            let finished = result(scores);
            assert!(state.finish_hand(&finished));
            assert!(!state.finish_hand(&finished));
            assert_eq!(
                state.scores()[0],
                i32::from(hand) * i32::from(mode.players() - 1)
            );
            assert_eq!(state.scores().iter().sum::<i32>(), 0);
            assert_eq!(state.complete(), hand == 5 * mode.players());
            assert_eq!(state.next_hand(), hand < 5 * mode.players());
        }
    }
    for rounds in [0, 6] {
        assert!(matches!(
            DoudizhuMatchState::new(
                DoudizhuRuleSet {
                    rounds,
                    ..Default::default()
                },
                PlayerId(0)
            ),
            Err(DoudizhuGameError::InvalidRules)
        ));
    }
}

#[test]
fn match_awards_use_total_scores_and_configured_round_brackets() {
    for (mode, short, long) in [
        (DoudizhuMode::Three, vec![3, 0, -3], vec![5, 0, -5]),
        (DoudizhuMode::Four, vec![4, 2, -2, -4], vec![6, 3, -3, -6]),
    ] {
        for rounds in 1..=5 {
            let mut state = DoudizhuMatchState::new(
                DoudizhuRuleSet {
                    mode,
                    rounds,
                    ..DoudizhuRuleSet::default()
                },
                PlayerId(0),
            )
            .unwrap();
            for _ in 0..state.progress().total_hands() {
                state.finish_hand(&result(
                    (0..mode.players()).map(|i| -i32::from(i)).collect(),
                ));
                state.next_hand();
            }
            assert_eq!(
                state.reference_deltas(mode).unwrap(),
                if rounds <= 2 {
                    short.clone()
                } else {
                    long.clone()
                }
            );
        }
    }
    let mut tied = DoudizhuMatchState::new(DoudizhuRuleSet::default(), PlayerId(0)).unwrap();
    for _ in 0..3 {
        tied.finish_hand(&result(vec![1, 1, -2]));
        tied.next_hand();
    }
    assert_eq!(
        tied.reference_deltas(DoudizhuMode::Three).unwrap(),
        [3, 3, -3]
    );
}

#[test]
fn four_player_small_bombs_do_not_double_but_six_cards_and_rockets_do() {
    for (mode, ranks, multiplier) in [
        (DoudizhuMode::Three, vec![0; 4], 2),
        (DoudizhuMode::Three, vec![13, 14], 2),
        (DoudizhuMode::Four, vec![0; 4], 1),
        (DoudizhuMode::Four, vec![0; 5], 1),
        (DoudizhuMode::Four, vec![0; 6], 2),
        (DoudizhuMode::Four, vec![0; 8], 2),
        (DoudizhuMode::Four, vec![13, 13, 14, 14], 2),
    ] {
        let wanted = cards(&ranks);
        let deck = build_deck(mode);
        let mut rest = deck.iter().copied().filter(|card| !wanted.contains(card));
        let n = usize::from(mode.players());
        let arranged = (0..deck.len())
            .map(|i| {
                if i % n == 0 && i / n < wanted.len() {
                    wanted[i / n]
                } else {
                    rest.next().unwrap()
                }
            })
            .collect();
        let mut game = DoudizhuGameState::new(
            DoudizhuRuleSet {
                mode,
                ..Default::default()
            },
            arranged,
            PlayerId(0),
        )
        .unwrap();
        game.act(PlayerId(0), DoudizhuAction::Bid(3)).unwrap();
        game.act(PlayerId(0), DoudizhuAction::Play(wanted)).unwrap();
        assert_eq!(game.multiplier, multiplier);
    }
}
