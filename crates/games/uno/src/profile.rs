//! Long-term statistics owned by this game. Field order is part of the saved profile format.

use crate::UnoRuleSet;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct UnoProfileStats {
    pub completed_games: u32,
    pub total_reference_delta: i64,
    pub total_remaining_score: u64,
    pub placement_counts: [u32; UnoRuleSet::MAX_PLAYERS as usize],
    pub max_hand_cards: u16,
    pub max_penalty_cards: u16,
    pub max_skipped_turns: u16,
    pub uno_calls: u32,
    pub uno_penalties: u32,
    pub challenges: u32,
    pub successful_challenges: u32,
    pub challenges_received: u32,
    /// 别的玩家成功质疑自己的次数。
    pub successful_challenges_received: u32,
    /// 玩家实际收到抢出候选牌的次数，而非发送抢出命令的次数。
    #[cfg_attr(feature = "serde", serde(alias = "jump_in_attempts"))]
    pub jump_in_opportunities: u32,
    pub successful_jump_ins: u32,
}
