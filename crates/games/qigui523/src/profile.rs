//! Long-term statistics owned by this game. Field order is part of the saved profile format.

use leocard_game_common::{
    ActionTimeProfile, ActionTimeStats, GameCountStats, PlacementStats, ReferenceStats, ResultScope,
};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct QiGui523ProfileStats {
    pub completed_games: u32,
    pub total_score: u64,
    pub total_reference_delta: i64,
    pub placement_counts: [u32; 6],
    pub straight_plays: u32,
    pub consecutive_pair_plays: u32,
    pub airplane_plays: u32,
    pub bomb_plays: u32,
    pub heaven_bomb_plays: u32,
    pub longest_straight: u16,
    pub longest_consecutive_pairs: u16,
    pub longest_airplane: u16,
    pub action_time: ActionTimeStats,
}

impl ActionTimeProfile for QiGui523ProfileStats {
    fn action_time_stats(&self) -> &ActionTimeStats {
        &self.action_time
    }

    fn action_time_stats_mut(&mut self) -> &mut ActionTimeStats {
        &mut self.action_time
    }
}

impl GameCountStats for QiGui523ProfileStats {
    const COMPLETION_SCOPE: ResultScope = ResultScope::Match;

    fn completed_games(&self) -> u32 {
        self.completed_games
    }

    fn completed_games_mut(&mut self) -> &mut u32 {
        &mut self.completed_games
    }
}

impl ReferenceStats for QiGui523ProfileStats {
    fn total_reference_delta(&self) -> i64 {
        self.total_reference_delta
    }

    fn total_reference_delta_mut(&mut self) -> &mut i64 {
        &mut self.total_reference_delta
    }
}

impl PlacementStats for QiGui523ProfileStats {
    const PLACEMENT_SCOPE: ResultScope = ResultScope::Match;

    fn placement_counts(&self) -> &[u32] {
        &self.placement_counts
    }

    fn placement_counts_mut(&mut self) -> &mut [u32] {
        &mut self.placement_counts
    }
}
