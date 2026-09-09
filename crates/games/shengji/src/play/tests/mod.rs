mod following;
mod scoring;
mod structures;

use super::*;
use crate::{ShengjiRuleSet, ShengjiThrowPenalty, ShengjiTrump};

fn card(deck: u8, suit: ShengjiSuit, rank: ShengjiRank) -> ShengjiCard {
    ShengjiCard::suited(deck, suit, rank)
}

fn pair(suit: ShengjiSuit, rank: ShengjiRank) -> [ShengjiCard; 2] {
    [card(0, suit, rank), card(1, suit, rank)]
}

fn triple(suit: ShengjiSuit, rank: ShengjiRank) -> [ShengjiCard; 3] {
    [
        card(0, suit, rank),
        card(1, suit, rank),
        card(2, suit, rank),
    ]
}

fn quad(suit: ShengjiSuit, rank: ShengjiRank) -> [ShengjiCard; 4] {
    [
        card(0, suit, rank),
        card(1, suit, rank),
        card(2, suit, rank),
        card(3, suit, rank),
    ]
}

fn trump() -> ShengjiTrump {
    ShengjiTrump::new(ShengjiRank::Ten, Some(ShengjiSuit::Heart)).unwrap()
}
