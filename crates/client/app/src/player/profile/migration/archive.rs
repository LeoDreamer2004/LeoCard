//! 新增游戏档案前的公共存档布局，保留已有游戏统计的原始字段顺序。

use super::super::{StoredGameProfiles, StoredPlayerProfile, StoredRatingProfile};
use super::profile::decode_exact;
use leocard_protocol::{
    BlackjackProfileStats, MahjongProfileStats, MinecraftGuessProfileStats, PlayerInteractionStats,
    ShengjiProfileStats, TexasHoldemProfileStats, UnoProfileStats,
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

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

pub(super) fn decode_archive<T: DeserializeOwned + Into<TexasHoldemProfileStats>>(
    bytes: &[u8],
) -> Result<StoredPlayerProfile, String> {
    let previous: ArchivedPlayerProfile<T> =
        decode_exact(bytes).map_err(|error| format!("玩家档案已损坏：{error}"))?;
    Ok(StoredPlayerProfile {
        secret_key: previous.secret_key,
        games: StoredGameProfiles {
            doudizhu_stats: None,
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
