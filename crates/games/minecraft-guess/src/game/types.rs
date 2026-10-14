use super::GuessRecord;
use crate::{Entry, EntryKind, GuessMode};
use serde::{Deserialize, Serialize};

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
