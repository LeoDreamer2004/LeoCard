//! Long-term statistics owned by this game. Field order is part of the saved profile format.

use crate::{McGuessCategoryStatistics, McGuessRuleSet};
use leocard_game_common::{
    ActionTimeProfile, ActionTimeStats, GameCountStats, PlacementStats, ReferenceStats, ResultScope,
};
use serde::{Deserialize, Serialize};

/// Long-term records from completed MC guessing matches.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct McGuessProfileStats {
    pub completed_games: u32,
    pub total_reference_delta: i64,
    pub total_score: u64,
    pub placement_counts: [u32; McGuessRuleSet::MAX_PLAYERS as usize],
    /// Completed questions and guessable graph nodes, including unsolved ones.
    pub categories: McGuessCategoryStatistics,
    pub known_guesses: u32,
    /// Known guesses at least as close as the previous known guess; the first qualifies.
    pub non_worse_guesses: u32,
    pub unknown_guesses: u32,
    /// Accepted guesses only in games where unknown guesses are enabled; skips excluded.
    pub unknown_enabled_guesses: u32,
    pub skips: u32,
    pub action_time: ActionTimeStats,
}

impl McGuessProfileStats {
    pub fn small_games(&self) -> u32 {
        self.categories.total().games
    }

    pub fn wins(&self) -> u32 {
        self.categories.total().wins
    }

    pub fn actions(&self) -> u32 {
        self.known_guesses
            .saturating_add(self.unknown_guesses)
            .saturating_add(self.skips)
    }
}

impl ActionTimeProfile for McGuessProfileStats {
    fn action_time_stats(&self) -> &ActionTimeStats {
        &self.action_time
    }

    fn action_time_stats_mut(&mut self) -> &mut ActionTimeStats {
        &mut self.action_time
    }
}

impl GameCountStats for McGuessProfileStats {
    const COMPLETION_SCOPE: ResultScope = ResultScope::Match;

    fn completed_games(&self) -> u32 {
        self.completed_games
    }

    fn completed_games_mut(&mut self) -> &mut u32 {
        &mut self.completed_games
    }
}

impl ReferenceStats for McGuessProfileStats {
    fn total_reference_delta(&self) -> i64 {
        self.total_reference_delta
    }

    fn total_reference_delta_mut(&mut self) -> &mut i64 {
        &mut self.total_reference_delta
    }
}

impl PlacementStats for McGuessProfileStats {
    const PLACEMENT_SCOPE: ResultScope = ResultScope::Match;

    fn placement_counts(&self) -> &[u32] {
        &self.placement_counts
    }

    fn placement_counts_mut(&mut self) -> &mut [u32] {
        &mut self.placement_counts
    }
}
