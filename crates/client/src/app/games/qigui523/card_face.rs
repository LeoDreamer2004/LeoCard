//! 本游戏的牌数据到公共印刷牌面的适配。

use crate::app::runtime::{CardFace, CardRank, CardSuit};
use leocard_qigui523::{QiGuiCard, QiGuiRank, QiGuiSuit};

impl From<QiGuiSuit> for CardSuit {
    fn from(suit: QiGuiSuit) -> Self {
        match suit {
            QiGuiSuit::Diamond => Self::Diamond,
            QiGuiSuit::Club => Self::Club,
            QiGuiSuit::Heart => Self::Heart,
            QiGuiSuit::Spade => Self::Spade,
        }
    }
}

impl From<QiGuiCard> for CardFace {
    fn from(card: QiGuiCard) -> Self {
        let rank = match card.rank() {
            QiGuiRank::Ace => CardRank::Ace,
            QiGuiRank::Two => CardRank::Two,
            QiGuiRank::Three => CardRank::Three,
            QiGuiRank::Four => CardRank::Four,
            QiGuiRank::Five => CardRank::Five,
            QiGuiRank::Six => CardRank::Six,
            QiGuiRank::Seven => CardRank::Seven,
            QiGuiRank::Eight => CardRank::Eight,
            QiGuiRank::Nine => CardRank::Nine,
            QiGuiRank::Ten => CardRank::Ten,
            QiGuiRank::Jack => CardRank::Jack,
            QiGuiRank::Queen => CardRank::Queen,
            QiGuiRank::King => CardRank::King,
            QiGuiRank::Joker => {
                return if card.suit() == QiGuiSuit::Spade {
                    Self::BigJoker
                } else {
                    Self::SmallJoker
                };
            }
        };
        Self::Suited {
            rank,
            suit: card.suit().into(),
        }
    }
}
