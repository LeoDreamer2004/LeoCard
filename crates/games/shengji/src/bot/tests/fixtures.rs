use crate::{
    ShengjiBiddingObservation, ShengjiBot, ShengjiBotAction, ShengjiBotObservation,
    ShengjiGameState, ShengjiMode, ShengjiRuleSet, ShengjiTeamProgress, build_deck_for,
};
use leocard_game_common::{PlayerId, SearchBudget, SearchRandom};
use std::time::Duration;

pub(super) fn before_burial(mode: ShengjiMode, seed: u64) -> ShengjiGameState {
    let rules = ShengjiRuleSet::for_mode(mode);
    let mut deck = build_deck_for(rules.deck_count);
    let mut random = SearchRandom::new(seed);
    for index in (1..deck.len()).rev() {
        deck.swap(index, random.index(index + 1));
    }
    let mut game = ShengjiGameState::new(
        rules,
        ShengjiTeamProgress::for_rules(&rules),
        Some(PlayerId(0)),
        PlayerId(0),
        deck,
    )
    .unwrap();
    game.deal_all().unwrap();
    for player in 0..rules.player_count() {
        let player = PlayerId(player);
        if let Some(cards) =
            ShengjiBot::declaration(&ShengjiBiddingObservation::from_game(&game, player).unwrap())
        {
            game.declare(player, &cards).unwrap();
        }
    }
    game.close_bidding_and_take_kitty().unwrap();
    game
}

pub(super) fn game(mode: ShengjiMode, seed: u64) -> ShengjiGameState {
    let mut game = before_burial(mode, seed);
    let player = game.dealer().unwrap();
    let ShengjiBotAction::Bury(cards) = ShengjiBot::choose(
        &ShengjiBotObservation::from_game(&game, player).unwrap(),
        &SearchBudget::new(Duration::ZERO),
    ) else {
        panic!("bury")
    };
    game.bury(player, &cards).unwrap();
    game
}
