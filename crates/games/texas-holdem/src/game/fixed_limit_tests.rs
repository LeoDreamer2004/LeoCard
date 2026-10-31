use super::{TexasAction, TexasGameError, TexasGameState, TexasPhase, TexasStreet};
use crate::{TexasBot, TexasBotObservation, TexasRuleSet, build_deck};
use leocard_game_common::{PlayerId, SearchBudget};

use std::time::Duration;

fn game(stacks: Vec<u32>) -> TexasGameState {
    let mut state = TexasGameState::new_with_stacks(
        TexasRuleSet {
            player_count: 3,
            starting_chips: 50,
            fixed_limit: true,
            ..Default::default()
        },
        PlayerId(0),
        stacks,
        build_deck(false),
    )
    .unwrap();
    for _ in 0..2 {
        state
            .act(state.current_player().unwrap(), TexasAction::PostBlind)
            .unwrap();
    }
    state
}

#[test]
fn fixed_amounts_cap_raises_and_reset_on_each_street() {
    let mut state = game(vec![50; 3]);
    for target in [4, 6, 8] {
        let player = state.current_player().unwrap();
        assert_eq!(state.raise_sizing(player).unwrap().maximum(), Some(target));
        state.act(player, TexasAction::RaiseTo(target)).unwrap();
    }
    let player = state.current_player().unwrap();
    assert!(!state.raise_allowed(player).unwrap());
    assert!(!state.all_in_allowed(player).unwrap());
    assert_eq!(state.raise_sizing(player).unwrap().maximum(), None);
    let before = state.clone();
    for action in [TexasAction::RaiseTo(10), TexasAction::AllIn] {
        assert_eq!(
            state.act(player, action),
            Err(TexasGameError::RaiseLimitReached)
        );
        assert_eq!(state, before);
    }
    for _ in 0..2 {
        state
            .act(state.current_player().unwrap(), TexasAction::Call)
            .unwrap();
    }
    for (street, increment) in [
        (TexasStreet::Flop, 2),
        (TexasStreet::Turn, 4),
        (TexasStreet::River, 4),
    ] {
        assert_eq!(state.phase(), &TexasPhase::Betting(street));
        assert_eq!(state.minimum_raise(), increment);
        let player = state.current_player().unwrap();
        assert_eq!(
            state.raise_sizing(player).unwrap().maximum(),
            Some(increment)
        );
        state.act(player, TexasAction::RaiseTo(increment)).unwrap();
        assert_eq!(state.minimum_raise(), increment);
        for _ in 0..2 {
            state
                .act(state.current_player().unwrap(), TexasAction::Call)
                .unwrap();
        }
    }
    assert!(matches!(state.phase(), TexasPhase::Complete(_)));
    assert_eq!(
        state
            .players()
            .iter()
            .map(|player| player.stack())
            .sum::<u32>(),
        150
    );
}

#[test]
fn excessive_raises_and_all_ins_are_rejected_without_mutation() {
    let mut state = game(vec![50; 3]);
    let player = state.current_player().unwrap();
    let before = state.clone();
    for (action, target) in [(TexasAction::RaiseTo(6), 6), (TexasAction::AllIn, 50)] {
        assert_eq!(
            state.act(player, action),
            Err(TexasGameError::RaiseExceedsLimit {
                maximum_target: 4,
                target
            })
        );
        assert_eq!(state, before);
    }
}

#[test]
fn short_all_in_can_call_but_does_not_reopen_a_raise() {
    let mut state = game(vec![50, 50, 5]);
    state.act(PlayerId(0), TexasAction::RaiseTo(4)).unwrap();
    state.act(PlayerId(1), TexasAction::Call).unwrap();
    assert!(state.all_in_allowed(PlayerId(2)).unwrap());
    state.act(PlayerId(2), TexasAction::AllIn).unwrap();
    assert_eq!(state.current_bet(), 5);
    assert_eq!(state.minimum_raise(), 2);
    assert!(!state.raise_allowed(PlayerId(0)).unwrap());
    state.act(PlayerId(0), TexasAction::Call).unwrap();
    state.act(PlayerId(1), TexasAction::Call).unwrap();
    assert_eq!(state.phase(), &TexasPhase::Betting(TexasStreet::Flop));
    assert!(state.raise_allowed(PlayerId(1)).unwrap());
}

#[test]
fn bots_only_choose_legal_limit_actions() {
    for omaha in [false, true] {
        for offset in [0, 11, 23] {
            let mut deck = build_deck(false);
            deck.rotate_left(offset);
            let mut state = TexasGameState::new_with_deck(
                TexasRuleSet {
                    fixed_limit: true,
                    omaha,
                    starting_chips: 50,
                    ..Default::default()
                },
                PlayerId(0),
                deck,
            )
            .unwrap();
            for _ in 0..100 {
                let Some(player) = state.current_player() else {
                    break;
                };
                let observation = TexasBotObservation::from_game(&state, player).unwrap();
                let action =
                    TexasBot::choose(&observation, &SearchBudget::new(Duration::from_millis(1)));
                state.act(player, action).unwrap();
            }
            assert!(matches!(state.phase(), TexasPhase::Complete(_)));
        }
    }
}
