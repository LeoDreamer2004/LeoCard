use crate::{ShengjiCard, ShengjiRank, ShengjiSuit};

use crate::ShengjiTrump;

pub(super) fn card(deck: u8, suit: ShengjiSuit, rank: ShengjiRank) -> ShengjiCard {
    ShengjiCard::suited(deck, suit, rank)
}

pub(super) fn pair(suit: ShengjiSuit, rank: ShengjiRank) -> [ShengjiCard; 2] {
    [card(0, suit, rank), card(1, suit, rank)]
}

pub(super) fn triple(suit: ShengjiSuit, rank: ShengjiRank) -> [ShengjiCard; 3] {
    [
        card(0, suit, rank),
        card(1, suit, rank),
        card(2, suit, rank),
    ]
}

pub(super) fn quad(suit: ShengjiSuit, rank: ShengjiRank) -> [ShengjiCard; 4] {
    [
        card(0, suit, rank),
        card(1, suit, rank),
        card(2, suit, rank),
        card(3, suit, rank),
    ]
}

pub(super) fn trump() -> ShengjiTrump {
    ShengjiTrump::new(ShengjiRank::Ten, Some(ShengjiSuit::Heart)).unwrap()
}
