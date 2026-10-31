//! Long-term statistics owned by this game. Field order is part of the saved profile format.

use crate::MahjongFan;

pub const MAHJONG_MAJOR_FANS: [MahjongFan; 15] = [
    MahjongFan::BigFourWinds,
    MahjongFan::BigThreeDragons,
    MahjongFan::AllGreen,
    MahjongFan::NineGates,
    MahjongFan::FourKongs,
    MahjongFan::SevenShiftedPairs,
    MahjongFan::ThirteenOrphans,
    MahjongFan::AllTerminals,
    MahjongFan::LittleFourWinds,
    MahjongFan::LittleThreeDragons,
    MahjongFan::AllHonors,
    MahjongFan::FourConcealedPungs,
    MahjongFan::PureTerminalChows,
    MahjongFan::QuadrupleChow,
    MahjongFan::FourPureShiftedPungs,
];

#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct MahjongProfileStats {
    pub completed_games: u32,
    pub total_reference_delta: i64,
    pub total_match_score: i64,
    pub placement_counts: [u32; 4],
    pub hands_played: u32,
    pub wins: u32,
    pub self_draws: u32,
    pub discards_into_win: u32,
    pub exhaustive_draws: u32,
    pub false_wins: u32,
    pub total_win_fan: u64,
    /// 按麻将规则库中 88、64、48 番番种的顺序排列。
    pub major_fan_counts: [u32; MAHJONG_MAJOR_FANS.len()],
}
