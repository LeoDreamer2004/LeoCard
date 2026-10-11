use crate::EntryKind;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CorrectGuess {
    pub id: String,
    pub kind: EntryKind,
}

/// Accepted live actions only. The answer ID appears only when guessed correctly.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GuessActionStatistics {
    pub player: u8,
    pub correct_entry: Option<CorrectGuess>,
    pub wrong_all_matching: bool,
    pub unknown: bool,
    pub unknown_run: u32,
    pub skip_run: u32,
    pub all_wrong_after_first: bool,
    pub ended_matching_run: bool,
    pub won_after_skip: bool,
    pub first_cycle_win: bool,
    pub opening_win: bool,
    pub opponents_all_skipped: bool,
    pub long_search_win: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GuessHandStatistics {
    pub player: u8,
    pub consistent: bool,
    pub round_sweep: bool,
    pub match_sweep: bool,
    pub no_answers_top: bool,
}

#[derive(Clone, Debug)]
pub(super) struct PlayerRoundProgress {
    pub player: u8,
    pub has_skipped: bool,
    pub known_guesses: u32,
    pub consistent: bool,
    pub unknown_run: u32,
    pub skip_run: u32,
}

impl PlayerRoundProgress {
    pub fn new(player: u8) -> Self {
        Self {
            player,
            has_skipped: false,
            known_guesses: 0,
            consistent: true,
            unknown_run: 0,
            skip_run: 0,
        }
    }
}
