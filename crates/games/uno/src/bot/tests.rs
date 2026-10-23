use super::{UnoBot, UnoBotObservation};
use crate::{GameState, UnoPlayerId, UnoRuleSet, build_deck};
use fastrand::Rng;

#[test]
fn changing_concealed_opponent_cards_does_not_change_the_bot_decision() {
    let mut deck = build_deck();
    let first = GameState::new_with_deck(UnoRuleSet::default(), 4, deck.clone()).unwrap();
    let player = first.turn().unwrap().current_player;
    let opponent = UnoPlayerId((player.0 + 1) % 4);
    deck.swap(opponent.0, 60);
    let mut second = GameState::new_with_deck(UnoRuleSet::default(), 4, deck).unwrap();
    assert_ne!(
        first.player(opponent).unwrap().hand(),
        second.player(opponent).unwrap().hand()
    );
    let observation = UnoBotObservation::from_game(&first, false).unwrap();
    let other = UnoBotObservation::from_game(&second, false).unwrap();
    assert_eq!(observation, other);
    let action = UnoBot::choose(&observation, &mut Rng::with_seed(7)).unwrap();
    assert_eq!(
        action,
        UnoBot::choose(&other, &mut Rng::with_seed(7)).unwrap()
    );
    action.apply(&mut second, player).unwrap();
}
