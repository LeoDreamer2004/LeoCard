use super::super::StoredPlayerProfile;
use super::archive::decode_archive;
use leocard_texas_holdem::TexasProfileStats;
use serde::{Deserialize, Serialize};

pub(in crate::player::profile::migration) const V3_PROFILE_HEADER: &[u8] = b"LeoCard/profile/v3\0";

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub(in crate::player::profile::migration) struct V3TexasStats {
    pub completed_games: u32,
    pub total_reference_delta: i64,
    pub total_final_chips: u64,
    pub placement_counts: [u32; 6],
    pub wagered_chips: u64,
    pub wager_actions: u32,
    pub voluntary_actions: u32,
    pub check_actions: u32,
    pub raise_actions: u32,
    pub all_in_actions: u32,
    pub hands_played: u32,
    pub hands_folded: u32,
    pub hand_category_counts: [u32; 10],
}

impl From<V3TexasStats> for TexasProfileStats {
    fn from(previous: V3TexasStats) -> Self {
        let mut placement_counts = [0; 8];
        placement_counts[..6].copy_from_slice(&previous.placement_counts);
        Self {
            completed_games: previous.completed_games,
            total_reference_delta: previous.total_reference_delta,
            total_final_chips: previous.total_final_chips,
            placement_counts,
            wagered_chips: previous.wagered_chips,
            wager_actions: previous.wager_actions,
            voluntary_actions: previous.voluntary_actions,
            check_actions: previous.check_actions,
            raise_actions: previous.raise_actions,
            all_in_actions: previous.all_in_actions,
            hands_played: previous.hands_played,
            hands_folded: previous.hands_folded,
            hand_category_counts: previous.hand_category_counts,
            entry_tracked_hands: 0,
            entered_hands: 0,
        }
    }
}

pub(super) fn decode_v3(bytes: &[u8]) -> Result<StoredPlayerProfile, String> {
    decode_archive::<V3TexasStats>(bytes)
}

pub(super) const V4_PROFILE_HEADER: &[u8] = b"LeoCard/profile/v4\0";

#[derive(Default, Deserialize, Serialize)]
struct V4TexasStats {
    base: V3TexasStats,
    entry_tracked_hands: u32,
    entered_hands: u32,
}

impl From<V4TexasStats> for TexasProfileStats {
    fn from(previous: V4TexasStats) -> Self {
        Self {
            entry_tracked_hands: previous.entry_tracked_hands,
            entered_hands: previous.entered_hands,
            ..previous.base.into()
        }
    }
}

pub(in crate::player::profile::migration) fn decode_v4(
    bytes: &[u8],
) -> Result<StoredPlayerProfile, String> {
    decode_archive::<V4TexasStats>(bytes)
}
