use crate::{Card, Mode};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PlayKind {
    Single,
    Pair,
    Triple,
    TripleSingle,
    TriplePair,
    Straight,
    Pairs,
    Airplane,
    AirplaneSingles,
    AirplanePairs,
    FourSingles,
    FourPairs,
    Bomb,
    Rocket,
}

impl PlayKind {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Single => "单张",
            Self::Pair => "对子",
            Self::Triple => "三张",
            Self::TripleSingle => "三带一",
            Self::TriplePair => "三带二",
            Self::Straight => "顺子",
            Self::Pairs => "连对",
            Self::Airplane => "飞机",
            Self::AirplaneSingles => "飞机带单",
            Self::AirplanePairs => "飞机带对",
            Self::FourSingles => "四带二",
            Self::FourPairs => "四带两对",
            Self::Bomb => "炸弹",
            Self::Rocket => "王炸",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Play {
    pub cards: Vec<Card>,
    pub kind: PlayKind,
    pub rank: u8,
}

impl Play {
    pub fn beats(&self, other: &Self) -> bool {
        if other.kind == PlayKind::Rocket {
            return false;
        }
        if self.kind == PlayKind::Rocket {
            return true;
        }
        if self.kind == PlayKind::Bomb {
            return other.kind != PlayKind::Bomb
                || (self.cards.len(), self.rank) > (other.cards.len(), other.rank);
        }
        other.kind != PlayKind::Bomb
            && self.kind == other.kind
            && self.cards.len() == other.cards.len()
            && self.rank > other.rank
    }

    pub fn doubles(&self, mode: Mode) -> bool {
        self.kind == PlayKind::Rocket
            || (self.kind == PlayKind::Bomb && (mode == Mode::Three || self.cards.len() >= 6))
    }
}
