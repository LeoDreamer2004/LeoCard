//! Long-term statistics owned by this game. Field order is part of the saved profile format.

#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct ShengjiProfileStats {
    pub completed_games: u32,
    pub total_reference_delta: i64,
    pub dealer_team_games: u32,
    pub dealer_team_score: u64,
    pub collecting_team_games: u32,
    pub collecting_team_score: u64,
    pub dealer_games: u32,
    pub declaration_games: u32,
    pub counter_games: u32,
    pub defended_kitty_games: u32,
    pub captured_kitty_games: u32,
    pub buried_games: u32,
    pub buried_points: u64,
    pub plays: u32,
    pub winning_plays: u32,
    pub crossing_games: u32,
    /// 拖拉机、泰坦尼克、炸弹、宇宙飞船、甩牌。
    pub play_category_counts: [u32; 5],
    pub longest_tractor: u16,
    pub longest_titanic: u16,
    pub longest_spaceship: u16,
    pub longest_throw: u16,
}
