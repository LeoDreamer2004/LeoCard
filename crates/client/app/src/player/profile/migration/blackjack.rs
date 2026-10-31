use super::super::{StoredGameProfiles, StoredPlayerProfile, StoredRatingProfile};
use super::mc_guess::PreviousMcGuessStats;
use super::profile::decode_exact;
use super::texas::V3TexasStats;
use leocard_mahjong::MahjongProfileStats;
use leocard_protocol::PlayerInteractionStats;
use leocard_shengji::ShengjiProfileStats;
use leocard_uno::UnoProfileStats;
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
    texas_stats: Option<V3TexasStats>,
    shengji_stats: Option<ShengjiProfileStats>,
    uno_stats: Option<UnoProfileStats>,
    interaction_stats: Option<PlayerInteractionStats>,
    mahjong_stats: Option<MahjongProfileStats>,
    mc_guess_stats: Option<PreviousMcGuessStats>,
}

pub(super) fn decode_v1(bytes: &[u8]) -> Result<StoredPlayerProfile, String> {
    let previous: V1PlayerProfile =
        decode_exact(bytes).map_err(|error| format!("玩家档案已损坏：{error}"))?;
    Ok(StoredPlayerProfile {
        secret_key: previous.secret_key,
        games: StoredGameProfiles {
            doudizhu_stats: None,
            qigui523: previous.games.qigui523,
            texas_stats: previous.games.texas_stats.map(Into::into),
            shengji_stats: previous.games.shengji_stats,
            uno_stats: previous.games.uno_stats,
            interaction_stats: previous.games.interaction_stats,
            mahjong_stats: previous.games.mahjong_stats,
            mc_guess_stats: previous.games.mc_guess_stats.map(Into::into),
            blackjack_stats: None,
        },
    })
}
