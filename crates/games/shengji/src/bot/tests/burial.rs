use super::fixtures::before_burial;
use crate::{ShengjiBot, ShengjiBotAction, ShengjiBotObservation, ShengjiMode};
use leocard_game_common::SearchBudget;
use std::cmp::Reverse;
use std::collections::HashSet;
use std::time::Duration;

#[test]
fn cancelled_burial_uses_a_whole_hand_plan_independent_of_hand_order() {
    for mode in [ShengjiMode::Classic, ShengjiMode::SixPlayer] {
        let mut game = before_burial(mode, 203);
        let player = game.dealer().unwrap();
        let mut observation = ShengjiBotObservation::from_game(&game, player).unwrap();
        // Put the most valuable control cards at the beginning, where the old
        // timeout path would bury them without evaluating the hand.
        observation
            .hand
            .sort_by_key(|card| Reverse(observation.trump.strength(*card)));
        let original_prefix = observation.hand[..game.rules().kitty_size()].to_vec();
        let budget = SearchBudget::new(Duration::ZERO);
        let action = ShengjiBot::choose(&observation, &budget);
        assert_eq!(action, observation.fallback());
        let ShengjiBotAction::Bury(cards) = &action else {
            panic!("burial")
        };
        assert_ne!(cards, &original_prefix);
        assert_eq!(cards.len(), cards.iter().collect::<HashSet<_>>().len());
        observation.hand.reverse();
        assert_eq!(ShengjiBot::choose(&observation, &budget), action);
        game.bury(player, cards).unwrap();
    }
}

#[test]
fn all_published_burials_are_legal_and_final_choice_is_the_latest() {
    for mode in [ShengjiMode::Classic, ShengjiMode::SixPlayer] {
        let game = before_burial(mode, 814);
        let player = game.dealer().unwrap();
        let observation = ShengjiBotObservation::from_game(&game, player).unwrap();
        let mut published = Vec::new();
        let final_choice = ShengjiBot::choose_with_progress(
            &observation,
            &SearchBudget::new(Duration::from_millis(300)),
            |choice| {
                let ShengjiBotAction::Bury(cards) = &choice else {
                    panic!("burial")
                };
                game.clone().bury(player, cards).unwrap();
                published.push(choice);
            },
        );
        assert_eq!(published.last(), Some(&final_choice));
        assert!(!published.is_empty());
    }
}
