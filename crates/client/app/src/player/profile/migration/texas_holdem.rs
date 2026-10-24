use super::super::{StoredGameProfiles, StoredPlayerProfile, StoredRatingProfile};
use super::profile::decode_exact;
use leocard_protocol::{
    BlackjackProfileStats, MahjongProfileStats, MinecraftGuessProfileStats, PlayerInteractionStats,
    ShengjiProfileStats, TexasHoldemProfileStats, UnoProfileStats,
};
use serde::{Deserialize, Serialize};

pub(super) const V3_PROFILE_HEADER: &[u8] = b"LeoCard/profile/v3\0";

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub(super) struct V3TexasHoldemStats {
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
        Self {
            completed_games: previous.completed_games,
            total_reference_delta: previous.total_reference_delta,
            total_final_chips: previous.total_final_chips,
            placement_counts: previous.placement_counts,
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
struct V3PlayerProfile {
    secret_key: [u8; 32],
    games: V3GameProfiles,
}

#[derive(Deserialize, Serialize)]
struct V3GameProfiles {
    qigui523: StoredRatingProfile,
    texas_holdem_stats: Option<V3TexasHoldemStats>,
    shengji_stats: Option<ShengjiProfileStats>,
    uno_stats: Option<UnoProfileStats>,
    interaction_stats: Option<PlayerInteractionStats>,
    mahjong_stats: Option<MahjongProfileStats>,
    minecraft_guess_stats: Option<MinecraftGuessProfileStats>,
    blackjack_stats: Option<BlackjackProfileStats>,
}

pub(super) fn decode_v3(bytes: &[u8]) -> Result<StoredPlayerProfile, String> {
    let previous: V3PlayerProfile =
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
