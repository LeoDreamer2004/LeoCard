use crate::{HandResult, ShengjiClassifiedPlay, ShengjiPlayerId, ShengjiRuleSet};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ShengjiOpeningHandStatistics {
    pub card_count: u8,
    pub trump_count: u8,
    pub joker_count: u8,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ShengjiBurialStatistics {
    pub card_count: u8,
    pub scoring_card_count: u8,
    pub points: u16,
}

/// Private facts are available only after a completed hand, never in a live snapshot.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ShengjiHandStatistics {
    pub player: ShengjiPlayerId,
    pub deck_count: u8,
    pub opening_hand: ShengjiOpeningHandStatistics,
    pub burial: Option<ShengjiBurialStatistics>,
    pub bottom_burier: Option<ShengjiPlayerId>,
    pub first_trick_cut_dealer: bool,
    pub result: HandResult,
    pub last_trick_winner: ShengjiPlayerId,
    pub last_winning_play: ShengjiClassifiedPlay,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ShengjiMatchStatistics {
    pub consecutive_dealer_shutouts: [u32; 2],
}

impl ShengjiMatchStatistics {
    pub fn record_hand(&mut self, result: &HandResult) {
        let team = usize::from(result.dealer_team.0);
        let previous = self.consecutive_dealer_shutouts[team];
        self.consecutive_dealer_shutouts = [0; 2];
        if result.collecting_score == 0 {
            self.consecutive_dealer_shutouts[team] = previous.saturating_add(1);
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ShengjiRedealReason {
    NoDeclaration,
    BottomFlipExhausted { power_outage_used: bool },
}

#[derive(Clone, Debug, Default)]
pub(in crate::game) struct HandStatisticsTracker {
    pub(in crate::game) opening:
        [Option<ShengjiOpeningHandStatistics>; ShengjiRuleSet::PLAYER_COUNT],
    pub(in crate::game) burial: [Option<ShengjiBurialStatistics>; ShengjiRuleSet::PLAYER_COUNT],
    pub(in crate::game) first_trick_cut_dealer: [bool; ShengjiRuleSet::PLAYER_COUNT],
}
