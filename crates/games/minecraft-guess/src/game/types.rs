use super::GuessRecord;
use crate::{Entry, EntryKind, GuessMode, MAX_GUESS_CHARACTERS};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum GameError {
    NotPlayersTurn,
    CannotSkip,
    Finished,
    UnknownEntry,
    AlreadyGuessed,
    InvalidGuess,
    GuessTooLong,
}

impl fmt::Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
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

/// The chosen answer is only included after the round ends.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RoundView {
    pub kind: EntryKind,
    pub character_count: u16,
    pub current_player: Option<u8>,
    pub guesses: Vec<GuessRecord>,
    pub skipped_players: Vec<u8>,
    pub answer: Option<Entry>,
    pub winner: Option<u8>,
}

impl RoundView {
    pub fn can_guess(&self, player: u8, mode: GuessMode) -> bool {
        !self.finished() && (mode == GuessMode::Race || self.current_player == Some(player))
    }

    pub fn finished(&self) -> bool {
        self.answer.is_some()
    }
}
