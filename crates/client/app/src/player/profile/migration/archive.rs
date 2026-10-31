//! 新增游戏档案前的公共存档布局，保留已有游戏统计的原始字段顺序。

use super::super::{StoredGameProfiles, StoredPlayerProfile, StoredRatingProfile};
use super::profile::decode_exact;
use leocard_blackjack::BlackjackProfileStats;
use leocard_mahjong::MahjongProfileStats;
use leocard_mc_guess::McGuessProfileStats;
use leocard_protocol::PlayerInteractionStats;
use leocard_shengji::ShengjiProfileStats;
use leocard_texas_holdem::TexasProfileStats;
use leocard_uno::UnoProfileStats;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

#[derive(Deserialize, Serialize)]
struct ArchivedPlayerProfile<T> {
    secret_key: [u8; 32],
    games: ArchivedGameProfiles<T>,
}

#[derive(Deserialize, Serialize)]
struct ArchivedGameProfiles<T> {
    qigui523: StoredRatingProfile,
    texas_stats: Option<T>,
    shengji_stats: Option<ShengjiProfileStats>,
    uno_stats: Option<UnoProfileStats>,
    interaction_stats: Option<PlayerInteractionStats>,
    mahjong_stats: Option<MahjongProfileStats>,
    mc_guess_stats: Option<McGuessProfileStats>,
    blackjack_stats: Option<BlackjackProfileStats>,
}

pub(super) fn decode_archive<T: DeserializeOwned + Into<TexasProfileStats>>(
    bytes: &[u8],
) -> Result<StoredPlayerProfile, String> {
    let previous: ArchivedPlayerProfile<T> =
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
            mc_guess_stats: previous.games.mc_guess_stats,
            blackjack_stats: previous.games.blackjack_stats,
        },
    })
}
