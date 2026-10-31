//! 本游戏的牌数据到公共印刷牌面的适配。

use crate::app::runtime::{CardFace, CardRank, CardSuit};
use leocard_texas_holdem::{TexasCard, TexasRank, TexasSuit};

impl From<TexasSuit> for CardSuit {
    fn from(suit: TexasSuit) -> Self {
        match suit {
            TexasSuit::Diamond => Self::Diamond,
            TexasSuit::Club => Self::Club,
            TexasSuit::Heart => Self::Heart,
            TexasSuit::Spade => Self::Spade,
        }
    }
}

impl From<TexasCard> for CardFace {
    fn from(card: TexasCard) -> Self {
        let rank = match card.rank() {
            TexasRank::Ace => CardRank::Ace,
            TexasRank::Two => CardRank::Two,
            TexasRank::Three => CardRank::Three,
            TexasRank::Four => CardRank::Four,
            TexasRank::Five => CardRank::Five,
            TexasRank::Six => CardRank::Six,
            TexasRank::Seven => CardRank::Seven,
            TexasRank::Eight => CardRank::Eight,
            TexasRank::Nine => CardRank::Nine,
            TexasRank::Ten => CardRank::Ten,
            TexasRank::Jack => CardRank::Jack,
            TexasRank::Queen => CardRank::Queen,
            TexasRank::King => CardRank::King,
        };
        Self::Suited {
            rank,
            suit: card.suit().into(),
        }
    }
}
