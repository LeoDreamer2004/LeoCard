//! Long-term statistics owned by this game. Field order is part of the saved profile format.

use crate::BlackjackPlayerStatistics;
use leocard_game_common::ActionTimeStats;
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
