use super::McGuessRecord;
use crate::{MAX_GUESS_CHARACTERS, McGuessEntry, McGuessEntryKind, McGuessMode};
use leocard_game_common::PlayerId;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum McGuessGameError {
    NotPlayersTurn,
    CannotSkip,
    Finished,
    UnknownEntry,
    AlreadyGuessed,
    InvalidGuess,
    GuessTooLong,
    GraphUnavailable,
    InvalidNode,
    NodeSwitchCoolingDown,
    CannotGuessTag,
    NodeAlreadyRevealed,
    ComboRequiresNeighbour,
}

impl fmt::Display for McGuessGameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GraphUnavailable => f.write_str("暂时未能生成足够的交织节点，请重试开始游戏"),
            Self::InvalidNode => f.write_str("请选择有效的图谱节点"),
            Self::NodeSwitchCoolingDown => f.write_str("请稍候再切换节点"),
            Self::CannotGuessTag => f.write_str("标签由相邻节点全部揭晓后自动解锁"),
            Self::NodeAlreadyRevealed => f.write_str("此节点已揭晓，请选择其他节点"),
            Self::ComboRequiresNeighbour => f.write_str("连击时请猜测刚猜中节点的相邻未揭晓节点"),
            Self::NotPlayersTurn => f.write_str("还没有轮到你猜测"),
            Self::CannotSkip => f.write_str("抢答模式不能跳过"),
            Self::Finished => f.write_str("本轮竞猜已经结束"),
            Self::UnknownEntry => f.write_str("没有找到这个名称，请输入完整的方块或物品名称"),
            Self::AlreadyGuessed => f.write_str("此条目已猜过"),
            Self::InvalidGuess => f.write_str("请输入有效的方块或物品名称"),
            Self::GuessTooLong => write!(f, "名称最多输入 {MAX_GUESS_CHARACTERS} 个字"),
        }
    }
}

impl Error for McGuessGameError {}

/// The chosen answer is only included after the round ends.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct McGuessRoundView {
    /// Completion is independent of a single answer: graph games have many answers.
    pub completed: bool,
    pub kind: McGuessEntryKind,
    pub character_count: u16,
    pub current_player: Option<PlayerId>,
    pub guesses: Vec<McGuessRecord>,
    pub skipped_players: Vec<PlayerId>,
    pub answer: Option<McGuessEntry>,
    pub winner: Option<PlayerId>,
}

impl McGuessRoundView {
    pub fn can_guess(&self, player: PlayerId, mode: McGuessMode) -> bool {
        !self.finished() && (mode == McGuessMode::Race || self.current_player == Some(player))
    }

    pub fn finished(&self) -> bool {
        self.completed
    }
}
