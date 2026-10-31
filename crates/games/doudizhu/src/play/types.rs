use crate::{DoudizhuCard, DoudizhuMode};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DoudizhuPlayKind {
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

impl DoudizhuPlayKind {
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
pub struct DoudizhuPlay {
    pub cards: Vec<DoudizhuCard>,
    pub kind: DoudizhuPlayKind,
    pub rank: u8,
}

impl DoudizhuPlay {
    pub fn beats(&self, other: &Self) -> bool {
        if other.kind == DoudizhuPlayKind::Rocket {
            return false;
        }
        if self.kind == DoudizhuPlayKind::Rocket {
            return true;
        }
        if self.kind == DoudizhuPlayKind::Bomb {
            return other.kind != DoudizhuPlayKind::Bomb
                || (self.cards.len(), self.rank) > (other.cards.len(), other.rank);
        }
        other.kind != DoudizhuPlayKind::Bomb
            && self.kind == other.kind
            && self.cards.len() == other.cards.len()
            && self.rank > other.rank
    }

    pub fn doubles(&self, mode: DoudizhuMode) -> bool {
        self.kind == DoudizhuPlayKind::Rocket
            || (self.kind == DoudizhuPlayKind::Bomb
                && (mode == DoudizhuMode::Three || self.cards.len() >= 6))
    }
}
