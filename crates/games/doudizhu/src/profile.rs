//! Long-term statistics owned by this game. Field order is part of the saved profile format.

/// 三人、四人斗地主共用的长期档案，场得分可为负数。
#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct DoudizhuProfileStats {
    pub completed_games: u32,
    pub total_reference_delta: i64,
    pub total_match_score: i64,
    pub hands_played: u32,
    pub landlord_hands: u32,
    /// 仅统计实际发生的叫分，不叫计零；含流局重发前的叫分。
    pub bid_actions: u32,
    pub total_bid_score: u64,
    pub straight_plays: u32,
    pub consecutive_pair_plays: u32,
    pub airplane_plays: u32,
    /// 普通炸弹、王炸、天王炸合并计数。
    pub bomb_plays: u32,
    pub longest_straight: u16,
    /// 连对按对数、飞机按主体的三张组数计长度，不包含翅膀。
    pub longest_consecutive_pairs: u16,
    pub longest_airplane: u16,
}
