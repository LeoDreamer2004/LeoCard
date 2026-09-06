use std::cmp::Ordering;
use std::collections::{BTreeMap, HashSet};
use std::fmt;

use crate::{ShengjiCard, ShengjiRank, ShengjiRuleSet, ShengjiSuit, ShengjiTrump};

#[path = "play_classification.rs"]
mod classification;
#[path = "play_follow.rs"]
mod follow;
#[path = "play_structure.rs"]
mod structure;

pub(crate) use classification::classify_cards;
use classification::*;
pub use classification::{category, compare_for_trick, strength};
use follow::special_follow_hierarchy;
#[cfg(test)]
use follow::{best_follow_tier, eight_card_tractor_hierarchy};
pub use follow::{classify_lead, follow_suggestions, forced_follow_cards, validate_follow};
use structure::*;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Category {
    Trump,
    Suit(ShengjiSuit),
    /// 仅用于缺门后的混合垫牌；首家不能领出混合门类，且这种牌永远不能赢墩。
    Mixed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Component {
    Single {
        card: ShengjiCard,
        strength: u8,
    },
    Pair {
        cards: [ShengjiCard; 2],
        strength: u8,
    },
    Triple {
        cards: [ShengjiCard; 3],
        strength: u8,
    },
    /// 四副牌中四张完全相同牌面的“炸弹”。相同牌力但不同花色不能合并。
    Quad {
        cards: [ShengjiCard; 4],
        strength: u8,
    },
    Tractor {
        cards: Vec<ShengjiCard>,
        pair_count: u8,
        top_strength: u8,
    },
    /// 三副牌中由相邻三同张组成的“泰坦尼克”。
    Titanic {
        cards: Vec<ShengjiCard>,
        triple_count: u8,
        top_strength: u8,
    },
    /// 四副牌中由相邻四同张组成的“宇宙飞船”。
    Spaceship {
        cards: Vec<ShengjiCard>,
        quad_count: u8,
        top_strength: u8,
    },
}

impl Component {
    pub fn card_count(&self) -> usize {
        match self {
            Self::Single { .. } => 1,
            Self::Pair { .. } => 2,
            Self::Triple { .. } => 3,
            Self::Quad { .. } => 4,
            Self::Tractor { cards, .. }
            | Self::Titanic { cards, .. }
            | Self::Spaceship { cards, .. } => cards.len(),
        }
    }

    pub const fn comparison_strength(&self) -> u8 {
        match self {
            Self::Single { strength, .. }
            | Self::Pair { strength, .. }
            | Self::Triple { strength, .. }
            | Self::Quad { strength, .. } => *strength,
            Self::Tractor { top_strength, .. }
            | Self::Titanic { top_strength, .. }
            | Self::Spaceship { top_strength, .. } => *top_strength,
        }
    }

    pub fn cards(&self) -> Vec<ShengjiCard> {
        match self {
            Self::Single { card, .. } => vec![*card],
            Self::Pair { cards, .. } => cards.to_vec(),
            Self::Triple { cards, .. } => cards.to_vec(),
            Self::Quad { cards, .. } => cards.to_vec(),
            Self::Tractor { cards, .. }
            | Self::Titanic { cards, .. }
            | Self::Spaceship { cards, .. } => cards.clone(),
        }
    }

    pub fn kitty_multiplier(&self) -> u32 {
        match self {
            Self::Single { .. } => 2,
            Self::Pair { .. } => 4,
            Self::Triple { .. } => 8,
            Self::Quad { .. } => 16,
            Self::Tractor { pair_count, .. } => 1_u32 << (u32::from(*pair_count) + 1),
            Self::Titanic { triple_count, .. } => 4_u32.saturating_pow(u32::from(*triple_count)),
            Self::Spaceship { quad_count, .. } => 8_u32.saturating_pow(u32::from(*quad_count)),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ShengjiClassifiedPlay {
    pub cards: Vec<ShengjiCard>,
    pub category: Category,
    pub components: Vec<Component>,
}

impl ShengjiClassifiedPlay {
    pub fn is_throw(&self) -> bool {
        self.components.len() > 1
            && !matches!(
                self.components.as_slice(),
                [Component::Quad { .. }, Component::Quad { .. }]
            )
    }

    pub fn strongest_component(&self) -> &Component {
        self.components
            .iter()
            .max_by_key(|component| component_priority(component))
            .expect("a play has at least one component")
    }

    pub fn kitty_multiplier(&self) -> u32 {
        self.components
            .iter()
            .map(Component::kitty_multiplier)
            .max()
            .unwrap_or(2)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ThrowFailure {
    pub attempted: ShengjiClassifiedPlay,
    pub forced: ShengjiClassifiedPlay,
    pub penalty_points: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TrickPlay {
    Accepted(ShengjiClassifiedPlay),
    ThrowFailed(ThrowFailure),
}

impl TrickPlay {
    pub fn effective(&self) -> &ShengjiClassifiedPlay {
        match self {
            Self::Accepted(play) => play,
            Self::ThrowFailed(failure) => &failure.forced,
        }
    }
}

#[derive(Clone, Debug)]
struct PairUnit {
    cards: [ShengjiCard; 2],
    strength: u8,
}

#[derive(Clone, Debug)]
struct TripleUnit {
    cards: [ShengjiCard; 3],
    strength: u8,
}

#[derive(Clone, Debug)]
struct QuadUnit {
    cards: [ShengjiCard; 4],
    strength: u8,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PlayError {
    Empty,
    DuplicatePhysicalCard,
    CardsNotOwned,
    MixedCategory,
    ThrowDisabled,
}

impl fmt::Display for PlayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("出牌不能为空"),
            Self::DuplicatePhysicalCard => f.write_str("同一张实体牌不能重复提交"),
            Self::CardsNotOwned => f.write_str("提交的牌不全在玩家手中"),
            Self::MixedCategory => f.write_str("首家只能出同一门副牌或全部主牌"),
            Self::ThrowDisabled => f.write_str("当前规则不允许甩牌"),
        }
    }
}

impl std::error::Error for PlayError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FollowError {
    Play(PlayError),
    WrongCardCount { expected: usize, actual: usize },
    MustFollowCategory { required: usize, actual: usize },
    MustFollowStructure,
}

impl fmt::Display for FollowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Play(error) => error.fmt(f),
            Self::WrongCardCount { expected, actual } => {
                write!(f, "必须跟出 {expected} 张牌，实际为 {actual}")
            }
            Self::MustFollowCategory { required, actual } => {
                write!(f, "必须跟出本门牌 {required} 张，实际为 {actual}")
            }
            Self::MustFollowStructure => {
                f.write_str("必须优先跟泰坦尼克、拖拉机、三同张和对子结构")
            }
        }
    }
}

impl std::error::Error for FollowError {}

#[cfg(test)]
#[path = "play_tests.rs"]
mod tests;
