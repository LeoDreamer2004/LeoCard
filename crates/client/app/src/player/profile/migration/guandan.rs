//! v7 档案增加掼蛋统计，保留身份、结算凭据、互动及全部旧游戏数据。

use super::super::{StoredGameProfiles, StoredPlayerProfile, StoredRatingProfile};
use super::profile::decode_exact;
use leocard_blackjack::BlackjackProfileStats;
use leocard_doudizhu::DoudizhuProfileStats;
use leocard_mahjong::MahjongProfileStats;
use leocard_mc_guess::McGuessProfileStats;
use leocard_protocol::PlayerInteractionStats;
use leocard_qigui523::QiGui523ProfileStats;
use leocard_shengji::ShengjiProfileStats;
use leocard_texas_holdem::TexasProfileStats;
use leocard_uno::UnoProfileStats;
use serde::Deserialize;

pub(super) const V7_PROFILE_HEADER: &[u8] = b"LeoCard/profile/v7\0";

#[derive(Deserialize)]
struct V7PlayerProfile {
    secret_key: [u8; 32],
    rating: StoredRatingProfile,
    interaction_stats: Option<PlayerInteractionStats>,
    games: V7GameProfiles,
}

#[derive(Deserialize)]
struct V7GameProfiles {
    qigui523_stats: Option<QiGui523ProfileStats>,
    texas_stats: Option<TexasProfileStats>,
    shengji_stats: Option<ShengjiProfileStats>,
    uno_stats: Option<UnoProfileStats>,
    mahjong_stats: Option<MahjongProfileStats>,
    mc_guess_stats: Option<McGuessProfileStats>,
    blackjack_stats: Option<BlackjackProfileStats>,
    doudizhu_stats: Option<DoudizhuProfileStats>,
}

pub(super) fn decode_v7(bytes: &[u8]) -> Result<StoredPlayerProfile, String> {
    let previous: V7PlayerProfile =
        decode_exact(bytes).map_err(|error| format!("玩家档案已损坏：{error}"))?;
    let games = previous.games;
    Ok(StoredPlayerProfile {
        secret_key: previous.secret_key,
        rating: previous.rating,
        interaction_stats: previous.interaction_stats,
        games: StoredGameProfiles {
            qigui523_stats: games.qigui523_stats,
            texas_stats: games.texas_stats,
            shengji_stats: games.shengji_stats,
            uno_stats: games.uno_stats,
            mahjong_stats: games.mahjong_stats,
            mc_guess_stats: games.mc_guess_stats,
            blackjack_stats: games.blackjack_stats,
            doudizhu_stats: games.doudizhu_stats,
            guandan_stats: None,
        },
        action_times: Default::default(),
    })
}

#[cfg(test)]
mod tests {
    use super::super::profile::decode_player_profile;
    use super::*;
    use crate::player::profile::{LocalPlayerProfile, PROFILE_HEADER};
    use leocard_guandan::GuandanProfileStats;
    use leocard_protocol::MatchId;
    use std::env;
    use std::fs;

    #[test]
    fn loading_v7_preserves_old_records_and_can_store_guandan() {
        let key = [37u8; 32];
        let receipt = MatchId([19; 16]);
        let rating = StoredRatingProfile {
            reference_points: -123,
            completed_games: 47,
            applied_matches: vec![receipt],
        };
        let interactions = Some(PlayerInteractionStats {
            flowers_received: 99,
            eggs_received: 21,
        });
        let qigui = Some(QiGui523ProfileStats {
            total_score: 840,
            ..Default::default()
        });
        let doudizhu = Some(DoudizhuProfileStats {
            completed_games: 5,
            bomb_plays: 3,
            ..Default::default()
        });
        let games = (
            &qigui,
            Some(TexasProfileStats::default()),
            Some(ShengjiProfileStats::default()),
            Some(UnoProfileStats::default()),
            Some(MahjongProfileStats::default()),
            Some(McGuessProfileStats::default()),
            Some(BlackjackProfileStats::default()),
            &doudizhu,
        );
        let payload = postcard::to_allocvec(&(key, &rating, &interactions, games)).unwrap();
        let mut old = V7_PROFILE_HEADER.to_vec();
        old.extend(&payload);
        let directory = env::temp_dir().join(format!(
            "leocard-guandan-profile-{:016x}",
            fastrand::u64(..)
        ));
        fs::create_dir(&directory).unwrap();
        let path = directory.join("profile.dat");
        fs::write(&path, &old).unwrap();
        let profile = LocalPlayerProfile::load_or_create_at(&path).unwrap();
        assert_eq!(profile.identity.secret_bytes(), key);
        assert_eq!(profile.reference_points(), rating.reference_points);
        assert!(profile.rating.applied_matches.contains(&receipt));
        assert_eq!(profile.game_profiles().qigui523, qigui);
        assert_eq!(profile.game_profiles().doudizhu, doudizhu);
        assert_eq!(profile.game_profiles().interactions, interactions);
        assert_eq!(profile.game_profiles().guandan, None);
        let mut expected = PROFILE_HEADER.to_vec();
        expected.extend(payload);
        expected.push(0); // New optional statistics follow the complete v7 payload.
        expected
            .extend(postcard::to_allocvec(&leocard_protocol::GameActionTimes::default()).unwrap());
        assert_eq!(fs::read(&path).unwrap(), expected);

        let mut stored = decode_player_profile(&expected).unwrap();
        stored.games.guandan_stats = Some(GuandanProfileStats {
            completed_games: 1,
            hands_played: 8,
            straight_flush_plays: 2,
            ..Default::default()
        });
        fs::write(&path, stored.encode().unwrap()).unwrap();
        let reloaded = LocalPlayerProfile::load_or_create_at(&path).unwrap();
        assert_eq!(reloaded.game_profiles().guandan, stored.games.guandan_stats);
        assert_eq!(reloaded.completed_games(), 47);
        assert_eq!(reloaded.identity.secret_bytes(), key);
        fs::remove_dir_all(directory).unwrap();
    }
}
