//! Long-term statistics owned by this game. Field order is part of the saved profile format.

use crate::BlackjackPlayerStatistics;
use leocard_game_common::{
    ActionTimeProfile, ActionTimeStats, GameCountStats, PlacementStats, ReferenceStats, ResultScope,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct BlackjackProfileStats {
    pub completed_games: u32,
    pub total_reference_delta: i64,
    pub total_chips_delta: i64,
    pub placement_counts: [u32; 6],
    pub statistics: BlackjackPlayerStatistics,
    pub action_time: ActionTimeStats,
}

impl ActionTimeProfile for BlackjackProfileStats {
    fn action_time_stats(&self) -> &ActionTimeStats {
        &self.action_time
    }

    fn action_time_stats_mut(&mut self) -> &mut ActionTimeStats {
        &mut self.action_time
    }
}

impl GameCountStats for BlackjackProfileStats {
    const COMPLETION_SCOPE: ResultScope = ResultScope::Match;

    fn completed_games(&self) -> u32 {
        self.completed_games
    }

    fn completed_games_mut(&mut self) -> &mut u32 {
        &mut self.completed_games
    }
}

impl ReferenceStats for BlackjackProfileStats {
    fn total_reference_delta(&self) -> i64 {
        self.total_reference_delta
    }

    fn total_reference_delta_mut(&mut self) -> &mut i64 {
        &mut self.total_reference_delta
    }
}

impl PlacementStats for BlackjackProfileStats {
    const PLACEMENT_SCOPE: ResultScope = ResultScope::Match;

    fn placement_counts(&self) -> &[u32] {
        &self.placement_counts
    }

    fn placement_counts_mut(&mut self) -> &mut [u32] {
        &mut self.placement_counts
    }
}
