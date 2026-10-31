use super::super::*;
use crate::{ShengjiCard, ShengjiRank, ShengjiRuleSet, ShengjiSuit, build_deck_for};
use leocard_game_common::PlayerId;

pub(super) fn dealt_game(rules: ShengjiRuleSet) -> ShengjiGameState {
    let mut deck = build_deck_for(rules.deck_count);
    // 保证 0 号玩家在第一张就拿到方块 2，可以确定性亮主。
    let target = ShengjiCard::suited(0, ShengjiSuit::Diamond, ShengjiRank::Two);
    let index = deck.iter().position(|card| *card == target).unwrap();
    deck.swap(0, index);
    let mut game = ShengjiGameState::new(
        rules,
        ShengjiTeamProgress::default(),
        None,
        PlayerId(0),
        deck,
    )
    .unwrap();
    game.deal_next().unwrap();
    game.declare(PlayerId(0), &[target]).unwrap();
    game.deal_all().unwrap();
    game.close_bidding_and_take_kitty().unwrap();
    game
}
