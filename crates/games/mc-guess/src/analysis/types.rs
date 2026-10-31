use super::McGuessPlayStatistics;
use crate::McGuessEntryKind;
use leocard_game_common::PlayerId;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct McGuessCorrectGuess {
    pub id: String,
    pub kind: McGuessEntryKind,
}

/// Accepted live actions only. The answer ID appears only when guessed correctly.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct McGuessActionStatistics {
    pub player: PlayerId,
    pub correct_entry: Option<McGuessCorrectGuess>,
    pub wrong_all_matching: bool,
    pub first_matching_wrong: bool,
    pub won_after_others_matching: bool,
    pub unlucky_matching: bool,
    pub wrong_length: bool,
    pub graph: Option<McGuessGraphActionStatistics>,
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

/// Facts specific to one graph; counters reset for each new graph.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct McGuessGraphActionStatistics {
    pub combo: u16,
    pub only_tag_neighbours: bool,
    pub opening_win: bool,
    pub first_green_nodes: u32,
    pub assisted_nodes: u32,
    pub combo_sweep: bool,
    pub round_win: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct McGuessHandStatistics {
    pub player: PlayerId,
    pub consistent: bool,
    pub round_sweep: bool,
    pub match_sweep: bool,
    pub no_answers_top: bool,
}

#[derive(Clone, Debug)]
pub(super) struct PlayerRoundProgress {
    pub player: PlayerId,
    pub has_skipped: bool,
    pub statistics: McGuessPlayStatistics,
    pub consistent: bool,
    pub unknown_run: u32,
    pub skip_run: u32,
}

impl PlayerRoundProgress {
    pub fn new(player: PlayerId) -> Self {
        Self {
            player,
            has_skipped: false,
            statistics: McGuessPlayStatistics::default(),
            consistent: true,
            unknown_run: 0,
            skip_run: 0,
        }
    }
}
