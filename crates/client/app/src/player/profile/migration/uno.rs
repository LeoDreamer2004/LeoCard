use leocard_uno::{UnoProfileStats, UnoRuleSet};

use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Deserialize, Serialize)]
pub(super) struct SixSeatUnoStats {
    pub completed_games: u32,
    pub total_reference_delta: i64,
    pub total_remaining_score: u64,
    pub placement_counts: [u32; 6],
    pub max_hand_cards: u16,
    pub max_penalty_cards: u16,
    pub max_skipped_turns: u16,
    pub uno_calls: u32,
    pub uno_penalties: u32,
    pub challenges: u32,
    pub successful_challenges: u32,
    pub challenges_received: u32,
    pub successful_challenges_received: u32,
    pub jump_in_opportunities: u32,
    pub successful_jump_ins: u32,
}

impl From<SixSeatUnoStats> for UnoProfileStats {
    fn from(previous: SixSeatUnoStats) -> Self {
        let mut placement_counts = [0; UnoRuleSet::MAX_PLAYERS as usize];
        placement_counts[..6].copy_from_slice(&previous.placement_counts);
        Self {
            completed_games: previous.completed_games,
            total_reference_delta: previous.total_reference_delta,
            total_remaining_score: previous.total_remaining_score,
            placement_counts,
            max_hand_cards: previous.max_hand_cards,
            max_penalty_cards: previous.max_penalty_cards,
            max_skipped_turns: previous.max_skipped_turns,
            uno_calls: previous.uno_calls,
            uno_penalties: previous.uno_penalties,
            challenges: previous.challenges,
            successful_challenges: previous.successful_challenges,
            challenges_received: previous.challenges_received,
            successful_challenges_received: previous.successful_challenges_received,
            jump_in_opportunities: previous.jump_in_opportunities,
            successful_jump_ins: previous.successful_jump_ins,
        }
    }
}
