use crate::{
    ShengjiBot, ShengjiBotObservation, ShengjiCard, ShengjiGameState, ShengjiMode, ShengjiRank,
    ShengjiRuleSet, ShengjiTeamProgress, build_deck_for,
};
use leocard_game_common::{PlayerId, SearchBudget};
use std::time::Duration;

#[test]
fn exchanging_hidden_hands_does_not_change_the_observation_or_rule_decision() {
    fn prepare(deck: Vec<ShengjiCard>, rules: ShengjiRuleSet) -> ShengjiGameState {
        let mut game = ShengjiGameState::new(
            rules,
            ShengjiTeamProgress::for_rules(&rules),
            Some(PlayerId(0)),
            PlayerId(0),
            deck,
        )
        .unwrap();
        game.deal_all().unwrap();
        let level = game.players()[0]
            .hand
            .iter()
            .find(|card| card.rank() == ShengjiRank::Two)
            .copied()
            .unwrap();
        game.declare(PlayerId(0), &[level]).unwrap();
        game.close_bidding_and_take_kitty().unwrap();
        let cards = game.players()[0].hand[..rules.kitty_size()].to_vec();
        game.bury(PlayerId(0), &cards).unwrap();
        game
    }
    let rules = ShengjiRuleSet::for_mode(ShengjiMode::SixPlayer);
    let mut deck = build_deck_for(rules.deck_count);
    let first = prepare(deck.clone(), rules);
    let left = first.players()[1]
        .hand
        .iter()
        .find(|card| card.rank() != ShengjiRank::Two)
        .unwrap();
    let right = first.players()[2]
        .hand
        .iter()
        .find(|card| card.rank() != ShengjiRank::Two)
        .unwrap();
    let a = deck.iter().position(|card| card == left).unwrap();
    let b = deck.iter().position(|card| card == right).unwrap();
    deck.swap(a, b);
    let second = prepare(deck, rules);
    assert_ne!(first.players()[1].hand, second.players()[1].hand);
    let observation = ShengjiBotObservation::from_game(&first, PlayerId(0)).unwrap();
    let changed = ShengjiBotObservation::from_game(&second, PlayerId(0)).unwrap();
    assert_eq!(observation, changed);
    assert_eq!(
        ShengjiBot::choose(&observation, &SearchBudget::new(Duration::ZERO)),
        ShengjiBot::choose(&changed, &SearchBudget::new(Duration::ZERO))
    );
}
