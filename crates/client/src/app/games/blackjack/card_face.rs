//! 本游戏的牌数据到公共印刷牌面的适配。

use crate::app::runtime::{CardFace, CardRank, CardSuit};
use leocard_blackjack::{BlackjackCard, BlackjackRank, BlackjackSuit};

impl From<BlackjackSuit> for CardSuit {
    fn from(suit: BlackjackSuit) -> Self {
        match suit {
            BlackjackSuit::Diamond => Self::Diamond,
            BlackjackSuit::Club => Self::Club,
            BlackjackSuit::Heart => Self::Heart,
            BlackjackSuit::Spade => Self::Spade,
        }
    }
}

impl From<BlackjackCard> for CardFace {
    fn from(card: BlackjackCard) -> Self {
        let rank = match card.rank {
            BlackjackRank::Ace => CardRank::Ace,
            BlackjackRank::Two => CardRank::Two,
            BlackjackRank::Three => CardRank::Three,
            BlackjackRank::Four => CardRank::Four,
            BlackjackRank::Five => CardRank::Five,
            BlackjackRank::Six => CardRank::Six,
            BlackjackRank::Seven => CardRank::Seven,
            BlackjackRank::Eight => CardRank::Eight,
            BlackjackRank::Nine => CardRank::Nine,
            BlackjackRank::Ten => CardRank::Ten,
            BlackjackRank::Jack => CardRank::Jack,
            BlackjackRank::Queen => CardRank::Queen,
            BlackjackRank::King => CardRank::King,
        };
        Self::Suited {
            rank,
            suit: card.suit.into(),
        }
    }
}
