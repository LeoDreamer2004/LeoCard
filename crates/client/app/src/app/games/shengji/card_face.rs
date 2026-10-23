//! 本游戏的牌数据到公共印刷牌面的适配。

use crate::app::runtime::{CardFace, CardRank, CardSuit};
use leocard_shengji::{ShengjiCard, ShengjiRank, ShengjiSuit};

impl From<ShengjiSuit> for CardSuit {
    fn from(suit: ShengjiSuit) -> Self {
        match suit {
            ShengjiSuit::Diamond => Self::Diamond,
            ShengjiSuit::Club => Self::Club,
            ShengjiSuit::Heart => Self::Heart,
            ShengjiSuit::Spade => Self::Spade,
        }
    }
}

impl From<ShengjiCard> for CardFace {
    fn from(card: ShengjiCard) -> Self {
        let rank = match card.rank() {
            ShengjiRank::Ace => CardRank::Ace,
            ShengjiRank::Two => CardRank::Two,
            ShengjiRank::Three => CardRank::Three,
            ShengjiRank::Four => CardRank::Four,
            ShengjiRank::Five => CardRank::Five,
            ShengjiRank::Six => CardRank::Six,
            ShengjiRank::Seven => CardRank::Seven,
            ShengjiRank::Eight => CardRank::Eight,
            ShengjiRank::Nine => CardRank::Nine,
            ShengjiRank::Ten => CardRank::Ten,
            ShengjiRank::Jack => CardRank::Jack,
            ShengjiRank::Queen => CardRank::Queen,
            ShengjiRank::King => CardRank::King,
            ShengjiRank::SmallJoker => return Self::SmallJoker,
            ShengjiRank::BigJoker => return Self::BigJoker,
        };
        Self::Suited {
            rank,
            suit: card
                .suit()
                .expect("a suited Shengji card has a suit")
                .into(),
        }
    }
}
