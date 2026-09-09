use crate::{MahjongPlayerId, MahjongTile, MahjongTileKind, RuleError, ScoreError};
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MahjongHandReplacementError {
    WrongTileCount {
        expected: u16,
        actual: u16,
    },
    FlowerNotAllowed {
        tile: MahjongTileKind,
    },
    TileUnavailable {
        tile: MahjongTileKind,
        requested: u16,
        available: u16,
    },
}

impl fmt::Display for MahjongHandReplacementError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WrongTileCount { expected, actual } => {
                write!(f, "手牌张数不对：需要 {expected} 张，实际输入 {actual} 张")
            }
            Self::FlowerNotAllowed { tile } => {
                write!(f, "开发者手牌不能包含花牌（{tile}）")
            }
            Self::TileUnavailable {
                tile,
                requested,
                available,
            } => write!(
                f,
                "{tile}存量不足：需要 {requested} 张，当前手牌和牌山中只有 {available} 张"
            ),
        }
    }
}

impl std::error::Error for MahjongHandReplacementError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GameError {
    InvalidRules(RuleError),
    InvalidDeckSize {
        expected: usize,
        actual: usize,
    },
    InvalidDeckContents,
    InvalidPlayer(MahjongPlayerId),
    NotPlayersTurn {
        expected: MahjongPlayerId,
        actual: MahjongPlayerId,
    },
    WrongPhase,
    TileNotInHand(MahjongTile),
    InvalidClaim,
    AlreadyResponded,
    CannotWin,
    CannotKong,
    InvalidHandReplacement(MahjongHandReplacementError),
    Score(ScoreError),
}

impl fmt::Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRules(error) => error.fmt(f),
            Self::InvalidDeckSize { expected, actual } => {
                write!(f, "牌墙张数错误：应为 {expected}，实际为 {actual}")
            }
            Self::InvalidDeckContents => f.write_str("牌墙不是完整且无重复的 144 张麻将牌"),
            Self::InvalidPlayer(player) => write!(f, "玩家 {:?} 不存在", player),
            Self::NotPlayersTurn { expected, actual } => {
                write!(f, "尚未轮到 {:?}，当前应由 {:?} 操作", actual, expected)
            }
            Self::WrongPhase => f.write_str("当前阶段不能执行该操作"),
            Self::TileNotInHand(tile) => {
                write!(f, "手中没有指定牌：{}#{}", tile.kind(), tile.copy())
            }
            Self::InvalidClaim => f.write_str("当前响应窗口不允许该操作"),
            Self::AlreadyResponded => f.write_str("已经提交过本次响应"),
            Self::CannotWin => f.write_str("当前手牌不能宣布和牌"),
            Self::CannotKong => f.write_str("当前不能开杠"),
            Self::InvalidHandReplacement(error) => error.fmt(f),
            Self::Score(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for GameError {}

impl From<RuleError> for GameError {
    fn from(value: RuleError) -> Self {
        Self::InvalidRules(value)
    }
}

impl From<ScoreError> for GameError {
    fn from(value: ScoreError) -> Self {
        Self::Score(value)
    }
}
