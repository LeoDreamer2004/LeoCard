//! 新增游戏档案前的公共存档布局，保留已有游戏统计的原始字段顺序。

use super::super::{StoredGameProfiles, StoredPlayerProfile, StoredRatingProfile};
use super::legacy::{
    BlackjackProfileStats, DoudizhuProfileStats, MahjongProfileStats, McGuessProfileStats,
    QiGui523ProfileStats, ShengjiProfileStats, TexasProfileStats, UnoProfileStats,
};
use super::profile::decode_exact;
use leocard_protocol::{MatchId, PlayerInteractionStats};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

pub(super) const V6_PROFILE_HEADER: &[u8] = b"LeoCard/profile/v6\0";

#[derive(Deserialize, Serialize)]
pub(super) struct V6PlayerProfile {
    pub(super) secret_key: [u8; 32],
    pub(super) games: V6GameProfiles,
}

#[derive(Deserialize, Serialize)]
pub(super) struct V6GameProfiles {
    pub(super) qigui523: LegacyRatingProfile,
    pub(super) texas_stats: Option<TexasProfileStats>,
    pub(super) shengji_stats: Option<ShengjiProfileStats>,
    pub(super) uno_stats: Option<UnoProfileStats>,
    pub(super) interaction_stats: Option<PlayerInteractionStats>,
    pub(super) mahjong_stats: Option<MahjongProfileStats>,
    pub(super) mc_guess_stats: Option<McGuessProfileStats>,
    pub(super) blackjack_stats: Option<BlackjackProfileStats>,
    pub(super) doudizhu_stats: Option<DoudizhuProfileStats>,
}

#[derive(Deserialize, Serialize)]
pub(super) struct LegacyRatingProfile {
    pub(super) reference_points: i32,
    pub(super) completed_games: u32,
    pub(super) applied_matches: Vec<MatchId>,
    pub(super) qigui523_stats: Option<QiGui523ProfileStats>,
}

impl From<V6PlayerProfile> for StoredPlayerProfile {
    fn from(previous: V6PlayerProfile) -> Self {
        let games = previous.games;
        Self {
            secret_key: previous.secret_key,
            rating: StoredRatingProfile {
                reference_points: games.qigui523.reference_points,
                completed_games: games.qigui523.completed_games,
                applied_matches: games.qigui523.applied_matches,
            },
            interaction_stats: games.interaction_stats,
            games: StoredGameProfiles {
                qigui523_stats: games.qigui523.qigui523_stats.map(Into::into),
                texas_stats: games.texas_stats.map(Into::into),
                shengji_stats: games.shengji_stats.map(Into::into),
                uno_stats: games.uno_stats.map(Into::into),
                mahjong_stats: games.mahjong_stats.map(Into::into),
                mc_guess_stats: games.mc_guess_stats.map(Into::into),
                blackjack_stats: games.blackjack_stats.map(Into::into),
                doudizhu_stats: games.doudizhu_stats.map(Into::into),
                guandan_stats: None,
                mahjong_link_stats: None,
            },
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
    qigui523: LegacyRatingProfile,
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
) -> Result<V6PlayerProfile, String> {
    let previous: ArchivedPlayerProfile<T> =
        decode_exact(bytes).map_err(|error| format!("玩家档案已损坏：{error}"))?;
    Ok(V6PlayerProfile {
        secret_key: previous.secret_key,
        games: V6GameProfiles {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::player::profile::{LocalPlayerProfile, PROFILE_HEADER};
    use std::env::temp_dir;
    use std::fs;

    #[test]
    fn loading_v6_rewrites_the_same_file_and_preserves_the_complete_profile() {
        let key = [37; 32];
        let receipt = MatchId([19; 16]);
        let rating = (-123i32, 47u32, vec![receipt]);
        let qigui = Some(QiGui523ProfileStats {
            total_score: 840,
            ..Default::default()
        });
        let texas = Some(TexasProfileStats {
            entered_hands: 13,
            ..Default::default()
        });
        let shengji = Some(ShengjiProfileStats::default());
        let uno = Some(UnoProfileStats::default());
        let interactions = Some(PlayerInteractionStats {
            flowers_received: 99,
            eggs_received: 21,
        });
        let mahjong = Some(MahjongProfileStats::default());
        let mc_guess = Some(McGuessProfileStats::default());
        let blackjack = Some(BlackjackProfileStats::default());
        let doudizhu = Some(DoudizhuProfileStats {
            bomb_plays: 3,
            ..Default::default()
        });
        // Postcard writes tuple fields consecutively, like struct fields.
        let mut old = V6_PROFILE_HEADER.to_vec();
        old.extend(
            postcard::to_allocvec(&(
                key,
                &rating,
                &qigui,
                &texas,
                &shengji,
                &uno,
                &interactions,
                &mahjong,
                &mc_guess,
                &blackjack,
                &doudizhu,
            ))
            .unwrap(),
        );
        let directory = temp_dir().join(format!("leocard-profile-{:016x}", fastrand::u64(..)));
        fs::create_dir(&directory).unwrap();
        let path = directory.join("profile.dat");
        let truncated = &old[..old.len() - 1];
        fs::write(&path, truncated).unwrap();
        assert!(LocalPlayerProfile::load_or_create_at(&path).is_err());
        assert_eq!(fs::read(&path).unwrap(), truncated);

        fs::write(&path, &old).unwrap();
        let profile = LocalPlayerProfile::load_or_create_at(&path).unwrap();
        assert_eq!(profile.identity.secret_bytes(), key);
        assert_eq!(profile.reference_points(), rating.0);
        assert_eq!(profile.completed_games(), rating.1);
        assert!(profile.rating.applied_matches.contains(&receipt));
        assert_eq!(profile.game_profiles().qigui523, qigui.map(Into::into));
        assert_eq!(profile.game_profiles().interactions, interactions);
        assert_eq!(profile.game_profiles().doudizhu, doudizhu.map(Into::into));
        let expected = fs::read(&path).unwrap();
        assert!(expected.starts_with(PROFILE_HEADER));
        assert!(LocalPlayerProfile::load_or_create_at(&path).is_ok());
        assert_eq!(fs::read(&path).unwrap(), expected);
        assert_eq!(fs::read_dir(&directory).unwrap().count(), 1);
        fs::remove_dir_all(directory).unwrap();
    }
}
