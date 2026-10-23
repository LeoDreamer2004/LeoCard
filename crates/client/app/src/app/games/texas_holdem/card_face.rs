//! 本游戏的牌数据到公共印刷牌面的适配。

use crate::app::runtime::{CardFace, CardRank, CardSuit};
use leocard_texas_holdem::{TexasHoldemCard, TexasHoldemRank, TexasHoldemSuit};

impl From<TexasHoldemSuit> for CardSuit {
    fn from(suit: TexasHoldemSuit) -> Self {
        match suit {
            TexasHoldemSuit::Diamond => Self::Diamond,
            TexasHoldemSuit::Club => Self::Club,
            TexasHoldemSuit::Heart => Self::Heart,
            TexasHoldemSuit::Spade => Self::Spade,
        }
    }
}

impl From<TexasHoldemCard> for CardFace {
    fn from(card: TexasHoldemCard) -> Self {
        let rank = match card.rank() {
            TexasHoldemRank::Ace => CardRank::Ace,
            TexasHoldemRank::Two => CardRank::Two,
            TexasHoldemRank::Three => CardRank::Three,
            TexasHoldemRank::Four => CardRank::Four,
            TexasHoldemRank::Five => CardRank::Five,
            TexasHoldemRank::Six => CardRank::Six,
            TexasHoldemRank::Seven => CardRank::Seven,
            TexasHoldemRank::Eight => CardRank::Eight,
            TexasHoldemRank::Nine => CardRank::Nine,
            TexasHoldemRank::Ten => CardRank::Ten,
            TexasHoldemRank::Jack => CardRank::Jack,
            TexasHoldemRank::Queen => CardRank::Queen,
            TexasHoldemRank::King => CardRank::King,
        };
        Self::Suited {
            rank,
            suit: card.suit().into(),
        }
    }
}
