mod bidding;
mod bottom;
mod crossing;
mod settlement;

use super::*;
use crate::play::classify_cards;
use crate::{
    ShengjiBidTrump, ShengjiCard, ShengjiPlayerId, ShengjiRank, ShengjiRuleSet, ShengjiSuit,
    ShengjiTeamId, ShengjiThrowPenalty, ShengjiTrump, build_deck, build_deck_for,
};

fn dealt_game(rules: ShengjiRuleSet) -> GameState {
    let mut deck = build_deck_for(rules.deck_count);
    // 保证 0 号玩家在第一张就拿到方块 2，可以确定性亮主。
    let target = ShengjiCard::suited(0, ShengjiSuit::Diamond, ShengjiRank::Two);
    let index = deck.iter().position(|card| *card == target).unwrap();
    deck.swap(0, index);
    let mut game = GameState::new(
        rules,
        TeamProgress::default(),
        None,
        ShengjiPlayerId(0),
        deck,
    )
    .unwrap();
    game.deal_next().unwrap();
    game.declare(ShengjiPlayerId(0), &[target]).unwrap();
    game.deal_all().unwrap();
    game.close_bidding_and_take_kitty().unwrap();
    game
}
