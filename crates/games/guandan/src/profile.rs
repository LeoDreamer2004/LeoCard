//! 掼蛋长期档案，整场结束后合并，小局游次仅统计已确定的名次。

use leocard_game_common::{
    ActionTimeProfile, ActionTimeStats, GameCountStats, PlacementStats, ReferenceStats, ResultScope,
};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct GuandanProfileStats {
    pub completed_games: u32,
    pub total_reference_delta: i64,
    pub match_wins: u32,
    pub hands_played: u32,
    pub hands_won: u32,
    pub placement_counts: [u32; 4],
    pub unranked_hands: u32,
    pub double_finish_hands: u32,
    pub levels_advanced: u64,
    pub straight_plays: u32,
    pub consecutive_pair_plays: u32,
    pub consecutive_triple_plays: u32,
    pub bomb_plays: u32,
    pub straight_flush_plays: u32,
    pub joker_bomb_plays: u32,
    pub action_time: ActionTimeStats,
}

impl ActionTimeProfile for GuandanProfileStats {
    fn action_time_stats(&self) -> &ActionTimeStats {
        &self.action_time
    }

    fn action_time_stats_mut(&mut self) -> &mut ActionTimeStats {
        &mut self.action_time
    }
}

impl GameCountStats for GuandanProfileStats {
    const COMPLETION_SCOPE: ResultScope = ResultScope::Match;

    fn completed_games(&self) -> u32 {
        self.completed_games
    }

    fn completed_games_mut(&mut self) -> &mut u32 {
        &mut self.completed_games
    }
}

impl ReferenceStats for GuandanProfileStats {
    fn total_reference_delta(&self) -> i64 {
        self.total_reference_delta
    }

    fn total_reference_delta_mut(&mut self) -> &mut i64 {
        &mut self.total_reference_delta
    }
}

impl PlacementStats for GuandanProfileStats {
    const PLACEMENT_SCOPE: ResultScope = ResultScope::Hand;

    fn placement_counts(&self) -> &[u32] {
        &self.placement_counts
    }

    fn placement_counts_mut(&mut self) -> &mut [u32] {
        &mut self.placement_counts
    }
}
