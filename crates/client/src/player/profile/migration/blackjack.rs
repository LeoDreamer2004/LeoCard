use super::super::{StoredGameProfiles, StoredPlayerProfile, StoredRatingProfile};
use super::minecraft_guess::PreviousMinecraftGuessStats;
use super::profile::decode_exact;
use leocard_protocol::{
    MahjongProfileStats, PlayerInteractionStats, ShengjiProfileStats, TexasHoldemProfileStats,
    UnoProfileStats,
};
use serde::{Deserialize, Serialize};

pub(super) const V1_PROFILE_HEADER: &[u8] = b"LeoCard/profile/v1\0";

#[derive(Deserialize, Serialize)]
struct V1PlayerProfile {
    secret_key: [u8; 32],
    games: V1GameProfiles,
}

#[derive(Deserialize, Serialize)]
struct V1GameProfiles {
    qigui523: StoredRatingProfile,
    texas_holdem_stats: Option<TexasHoldemProfileStats>,
    shengji_stats: Option<ShengjiProfileStats>,
    uno_stats: Option<UnoProfileStats>,
    interaction_stats: Option<PlayerInteractionStats>,
    mahjong_stats: Option<MahjongProfileStats>,
    minecraft_guess_stats: Option<PreviousMinecraftGuessStats>,
}

pub(super) fn decode_v1(bytes: &[u8]) -> Result<StoredPlayerProfile, String> {
    let previous: V1PlayerProfile =
        decode_exact(bytes).map_err(|error| format!("玩家档案已损坏：{error}"))?;
    Ok(StoredPlayerProfile {
        secret_key: previous.secret_key,
        games: StoredGameProfiles {
            qigui523: previous.games.qigui523,
            texas_holdem_stats: previous.games.texas_holdem_stats,
            shengji_stats: previous.games.shengji_stats,
            uno_stats: previous.games.uno_stats,
            interaction_stats: previous.games.interaction_stats,
            mahjong_stats: previous.games.mahjong_stats,
            minecraft_guess_stats: previous.games.minecraft_guess_stats.map(Into::into),
            blackjack_stats: None,
        },
    })
}
