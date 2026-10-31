//! Long-term statistics owned by this game. Field order is part of the saved profile format.

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
}
