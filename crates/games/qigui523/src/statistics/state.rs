use crate::{QiGuiPlayKind, QiGuiPlayerId, QiGuiRuleSet};

/// Match facts owned by the rule core; independent of persistence and achievements.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct QiGuiPlayerStatistics {
    pub plays: u32,
    pub straight_plays: u32,
    pub consecutive_pair_plays: u32,
    pub airplane_plays: u32,
    pub bomb_plays: u32,
    pub heaven_bomb_plays: u32,
    pub longest_straight: u16,
    pub longest_consecutive_pairs: u16,
    pub longest_airplane: u16,
    pub max_straight_run: u32,
    pub max_silent_tricks: u32,
    pub max_unanswered_tricks: u32,
    pub max_trick_points: u32,
    pub first_trick_with_points: bool,
    pub won_scoring_bomb: bool,
    pub full_high_hand: bool,
}

/// Private projection for one player, emitted only after an accepted action.
/// Captured card and point counts are deltas, not repeated match totals.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct QiGuiActionStatistics {
    pub player: QiGuiPlayerId,
    pub rules: QiGuiRuleSet,
    pub progress: QiGuiPlayerStatistics,
    pub played_kind: Option<QiGuiPlayKind>,
    pub played_count: u16,
    pub diamond_four: bool,
    pub spade_seven_follow: bool,
    pub heaven_over_heaven: bool,
    pub late_bomb: bool,
    pub straight_from_four: bool,
    pub full_hand_play: bool,
    pub bomb_revenge: bool,
    pub captured_fives: u32,
    pub captured_tens_and_kings: u32,
    pub captured_points: u32,
    pub completed_game: bool,
    pub won: bool,
    pub all_points: bool,
    pub comeback: bool,
    pub uncontested_win: bool,
}

#[derive(Clone, Debug, Default)]
pub(super) struct PlayerRuns {
    pub(super) straight: u32,
    pub(super) silent: u32,
    pub(super) unanswered: u32,
}

#[derive(Clone, Debug)]
pub struct QiGuiMatchStatistics {
    pub(super) players: Vec<QiGuiPlayerStatistics>,
    pub(super) runs: Vec<PlayerRuns>,
    pub(super) completed_tricks: u32,
    pub(super) total_points: u32,
    pub(super) finished: bool,
}
