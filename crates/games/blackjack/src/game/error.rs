#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum GameError {
    InvalidRules,
    InvalidPlayerCount,
    InvalidShoe,
    NotPlayersTurn,
    WrongPhase,
    InvalidBet,
    CannotDouble,
    CannotSplit,
    MatchComplete,
}

impl fmt::Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::NotPlayersTurn => "还没有轮到你行动",
            Self::WrongPhase => "当前阶段不能执行此操作",
            Self::InvalidBet => "下注须为偶数，且在最低下注与可用筹码范围内",
            Self::CannotDouble => "当前手牌或剩余筹码不允许加倍",
            Self::CannotSplit => "需要两张同点数牌、足够筹码，且不能超过四手",
            Self::MatchComplete => "整场游戏已经结束",
            Self::InvalidRules => "21 点规则配置无效",
            Self::InvalidPlayerCount => "21 点需要 1–6 名玩家",
            Self::InvalidShoe => "牌堆必须包含完整的六副扑克牌",
        })
    }
}

impl Error for GameError {}
