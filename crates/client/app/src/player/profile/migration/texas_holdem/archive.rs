use super::super::super::{StoredGameProfiles, StoredPlayerProfile, StoredRatingProfile};
use super::super::profile::decode_exact;
use leocard_protocol::{
    BlackjackProfileStats, MahjongProfileStats, MinecraftGuessProfileStats, PlayerInteractionStats,
    ShengjiProfileStats, TexasHoldemProfileStats, UnoProfileStats,
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

pub(in crate::player::profile::migration) const V3_PROFILE_HEADER: &[u8] = b"LeoCard/profile/v3\0";

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub(in crate::player::profile::migration) struct V3TexasHoldemStats {
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

impl From<V3TexasHoldemStats> for TexasHoldemProfileStats {
    fn from(previous: V3TexasHoldemStats) -> Self {
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

#[derive(Deserialize, Serialize)]
struct ArchivedPlayerProfile<T> {
    secret_key: [u8; 32],
    games: ArchivedGameProfiles<T>,
}

#[derive(Deserialize, Serialize)]
struct ArchivedGameProfiles<T> {
    qigui523: StoredRatingProfile,
    texas_holdem_stats: Option<T>,
    shengji_stats: Option<ShengjiProfileStats>,
    uno_stats: Option<UnoProfileStats>,
    interaction_stats: Option<PlayerInteractionStats>,
    mahjong_stats: Option<MahjongProfileStats>,
    minecraft_guess_stats: Option<MinecraftGuessProfileStats>,
    blackjack_stats: Option<BlackjackProfileStats>,
}

pub(in crate::player::profile::migration) fn decode_v3(
    bytes: &[u8],
) -> Result<StoredPlayerProfile, String> {
    decode_archive::<V3TexasHoldemStats>(bytes)
}

fn decode_archive<T: DeserializeOwned + Into<TexasHoldemProfileStats>>(
    bytes: &[u8],
) -> Result<StoredPlayerProfile, String> {
    let previous: ArchivedPlayerProfile<T> =
        decode_exact(bytes).map_err(|error| format!("玩家档案已损坏：{error}"))?;
    Ok(StoredPlayerProfile {
        secret_key: previous.secret_key,
        games: StoredGameProfiles {
            qigui523: previous.games.qigui523,
            texas_holdem_stats: previous.games.texas_holdem_stats.map(Into::into),
            shengji_stats: previous.games.shengji_stats,
            uno_stats: previous.games.uno_stats,
            interaction_stats: previous.games.interaction_stats,
            mahjong_stats: previous.games.mahjong_stats,
            minecraft_guess_stats: previous.games.minecraft_guess_stats,
            blackjack_stats: previous.games.blackjack_stats,
        },
    })
}

pub(in crate::player::profile::migration) const V4_PROFILE_HEADER: &[u8] = b"LeoCard/profile/v4\0";

#[derive(Default, Deserialize, Serialize)]
struct V4TexasHoldemStats {
    base: V3TexasHoldemStats,
    entry_tracked_hands: u32,
    entered_hands: u32,
}

impl From<V4TexasHoldemStats> for TexasHoldemProfileStats {
    fn from(previous: V4TexasHoldemStats) -> Self {
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
    decode_archive::<V4TexasHoldemStats>(bytes)
}
