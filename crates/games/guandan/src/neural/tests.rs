use super::{GuandanModelHistory, GuandanModelObservation};
use crate::{GuandanGameState, GuandanRuleSet, build_deck};
use leocard_game_common::PlayerId;

#[test]
fn model_observation_cannot_see_opponents_cards() {
    let mut deck = build_deck();
    let first =
        GuandanGameState::new(GuandanRuleSet::default(), deck.clone(), PlayerId(0)).unwrap();
    deck.swap(1, 2);
    deck.swap(5, 7);
    let second = GuandanGameState::new(GuandanRuleSet::default(), deck, PlayerId(0)).unwrap();
    let history = GuandanModelHistory::default();
    assert_ne!(
        first.player(PlayerId(1)).unwrap().hand(),
        second.player(PlayerId(1)).unwrap().hand()
    );
    let observation = GuandanModelObservation::from_game(&first, &history).unwrap();
    assert_eq!(
        Some(&observation),
        GuandanModelObservation::from_game(&second, &history).as_ref()
    );
    for (_, action) in observation.actions {
        first.clone().act(PlayerId(0), action).unwrap();
    }
}
