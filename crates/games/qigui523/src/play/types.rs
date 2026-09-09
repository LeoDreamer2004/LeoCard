use crate::{QiGuiCard, QiGuiRank};
use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum BombKind {
    OfAKind { card_count: usize, rank: QiGuiRank },
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum QiGuiPlayKind {
    Single,
    Pair,
    Straight { card_count: usize },
    ConsecutivePairs { pair_count: usize },
    Triple,
    TripleWithSingle,
    TripleWithPair,
    Airplane { triple_count: usize },
    Bomb(BombKind),
    HeavenBomb,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClassifiedPlay {
    pub(super) cards: Vec<QiGuiCard>,
    pub(super) kind: QiGuiPlayKind,
}

impl ClassifiedPlay {
    pub fn cards(&self) -> &[QiGuiCard] {
        &self.cards
    }

    pub fn kind(&self) -> &QiGuiPlayKind {
        &self.kind
    }

    pub fn score(&self) -> u16 {
        self.cards.iter().map(|card| card.score()).sum()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlayComparison {
    Greater,
    Equivalent,
    Lower,
    Incompatible,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PlayError {
    Empty,
    DuplicatePhysicalCard(QiGuiCard),
    CardOutsideConfiguredDeck(QiGuiCard),
    InvalidPattern,
}

impl fmt::Display for PlayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("出牌不能为空"),
            Self::DuplicatePhysicalCard(card) => write!(f, "同一张物理牌被重复提交：{card}"),
            Self::CardOutsideConfiguredDeck(card) => write!(f, "牌不属于当前配置的牌堆：{card}"),
            Self::InvalidPattern => f.write_str("这些牌不能组成合法牌型"),
        }
    }
}

impl std::error::Error for PlayError {}
