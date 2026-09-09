use crate::{BidError, FollowError, RuleError, ShengjiPlayerId};
use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GameError {
    Rules(RuleError),
    Bid(BidError),
    Play(crate::PlayError),
    Follow(FollowError),
    InvalidPlayer(ShengjiPlayerId),
    InvalidDeckSize {
        expected: usize,
        actual: usize,
    },
    InvalidDeckContents,
    WrongPhase,
    RedealRequired,
    NotDealer,
    WrongBuryCount {
        expected: usize,
        actual: usize,
    },
    CrossingNotEligible,
    CrossingAlreadyDecided,
    WrongCrossingCount {
        expected: usize,
        actual: usize,
    },
    CrossingMustIncludeAllTrumps,
    CrossingReturnNotRequired,
    CrossingAlreadyReturned,
    NotBottomCopyPlayer,
    CardsNotOwned,
    NotPlayersTurn {
        expected: ShengjiPlayerId,
        actual: ShengjiPlayerId,
    },
}

impl fmt::Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Rules(error) => error.fmt(f),
            Self::Bid(error) => error.fmt(f),
            Self::Play(error) => error.fmt(f),
            Self::Follow(error) => error.fmt(f),
            Self::InvalidPlayer(player) => write!(f, "玩家 {:?} 不存在", player),
            Self::InvalidDeckSize { expected, actual } => {
                write!(f, "牌堆应为 {expected} 张，实际为 {actual}")
            }
            Self::InvalidDeckContents => f.write_str("牌堆有缺牌、重复牌或非法牌"),
            Self::WrongPhase => f.write_str("当前阶段不能执行此操作"),
            Self::RedealRequired => f.write_str("无人亮主，必须重新发牌"),
            Self::NotDealer => f.write_str("只有庄家可以埋底"),
            Self::WrongBuryCount { expected, actual } => {
                write!(f, "必须埋 {expected} 张底牌，实际为 {actual}")
            }
            Self::CrossingNotEligible => f.write_str("你的主牌多于五张，不能五主过江"),
            Self::CrossingAlreadyDecided => f.write_str("你已经完成五主过江选择"),
            Self::WrongCrossingCount { expected, actual } => {
                write!(f, "五主过江必须选择 {expected} 张牌，实际为 {actual}")
            }
            Self::CrossingMustIncludeAllTrumps => f.write_str("五主过江必须选择手中的全部主牌"),
            Self::CrossingReturnNotRequired => f.write_str("当前不需要你归还过江牌"),
            Self::CrossingAlreadyReturned => f.write_str("你已经归还过江牌"),
            Self::NotBottomCopyPlayer => f.write_str("当前没有轮到该玩家抄底或重新埋底"),
            Self::CardsNotOwned => f.write_str("提交的牌不全在玩家手中"),
            Self::NotPlayersTurn { expected, actual } => {
                write!(f, "当前应由 {:?} 出牌，不是 {:?}", expected, actual)
            }
        }
    }
}

impl std::error::Error for GameError {}

impl From<RuleError> for GameError {
    fn from(value: RuleError) -> Self {
        Self::Rules(value)
    }
}

impl From<BidError> for GameError {
    fn from(value: BidError) -> Self {
        Self::Bid(value)
    }
}

impl From<crate::PlayError> for GameError {
    fn from(value: crate::PlayError) -> Self {
        Self::Play(value)
    }
}

impl From<FollowError> for GameError {
    fn from(value: FollowError) -> Self {
        Self::Follow(value)
    }
}
