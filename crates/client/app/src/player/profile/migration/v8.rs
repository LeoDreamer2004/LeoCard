//! Frozen v8 archive: preserve all nine game profiles before adding ten-game timing.
use super::super::{StoredGameProfiles, StoredPlayerProfile, StoredRatingProfile};
use super::legacy::{
    BlackjackProfileStats, DoudizhuProfileStats, GuandanProfileStats, MahjongProfileStats,
    McGuessProfileStats, QiGui523ProfileStats, ShengjiProfileStats, TexasProfileStats,
    UnoProfileStats,
};
use super::profile::decode_exact;
use leocard_protocol::PlayerInteractionStats;
use serde::Deserialize;

pub(super) const V8_PROFILE_HEADER: &[u8] = b"LeoCard/profile/v8\0";

#[derive(Deserialize)]
struct V8PlayerProfile {
    secret_key: [u8; 32],
    rating: StoredRatingProfile,
    interaction_stats: Option<PlayerInteractionStats>,
    games: V8GameProfiles,
}

#[derive(Deserialize)]
struct V8GameProfiles {
    qigui523_stats: Option<QiGui523ProfileStats>,
    texas_stats: Option<TexasProfileStats>,
    shengji_stats: Option<ShengjiProfileStats>,
    uno_stats: Option<UnoProfileStats>,
    mahjong_stats: Option<MahjongProfileStats>,
    mc_guess_stats: Option<McGuessProfileStats>,
    blackjack_stats: Option<BlackjackProfileStats>,
    doudizhu_stats: Option<DoudizhuProfileStats>,
    guandan_stats: Option<GuandanProfileStats>,
}

pub(super) fn decode_v8(bytes: &[u8]) -> Result<StoredPlayerProfile, String> {
    let previous: V8PlayerProfile =
        decode_exact(bytes).map_err(|error| format!("玩家档案已损坏：{error}"))?;
    let games = previous.games;
    Ok(StoredPlayerProfile {
        secret_key: previous.secret_key,
        rating: previous.rating,
        interaction_stats: previous.interaction_stats,
        games: StoredGameProfiles {
            qigui523_stats: games.qigui523_stats.map(Into::into),
            texas_stats: games.texas_stats.map(Into::into),
            shengji_stats: games.shengji_stats.map(Into::into),
            uno_stats: games.uno_stats.map(Into::into),
            mahjong_stats: games.mahjong_stats.map(Into::into),
            mc_guess_stats: games.mc_guess_stats.map(Into::into),
            blackjack_stats: games.blackjack_stats.map(Into::into),
            doudizhu_stats: games.doudizhu_stats.map(Into::into),
            guandan_stats: games.guandan_stats.map(Into::into),
            mahjong_link_stats: None,
        },
    })
}
