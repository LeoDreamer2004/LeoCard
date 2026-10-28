use super::super::{StoredGameProfiles, StoredPlayerProfile, StoredRatingProfile};
use super::minecraft_guess::PreviousMinecraftGuessStats;
use super::profile::decode_exact;
use super::texas_holdem::V3TexasHoldemStats;
use leocard_protocol::{
    BlackjackProfileStats, MahjongProfileStats, PlayerInteractionStats, ShengjiProfileStats,
    UnoProfileStats,
};
use serde::{Deserialize, Serialize};

pub(super) const V2_PROFILE_HEADER: &[u8] = b"LeoCard/profile/v2\0";

#[derive(Deserialize, Serialize)]
struct V2PlayerProfile {
    secret_key: [u8; 32],
    games: V2GameProfiles,
}

#[derive(Deserialize, Serialize)]
struct V2GameProfiles {
    qigui523: StoredRatingProfile,
    texas_holdem_stats: Option<V3TexasHoldemStats>,
    shengji_stats: Option<ShengjiProfileStats>,
    uno_stats: Option<UnoProfileStats>,
    interaction_stats: Option<PlayerInteractionStats>,
    mahjong_stats: Option<MahjongProfileStats>,
    minecraft_guess_stats: Option<PreviousMinecraftGuessStats>,
    blackjack_stats: Option<BlackjackProfileStats>,
}

pub(super) fn decode_v2(bytes: &[u8]) -> Result<StoredPlayerProfile, String> {
    let previous: V2PlayerProfile =
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
            minecraft_guess_stats: previous.games.minecraft_guess_stats.map(Into::into),
            blackjack_stats: previous.games.blackjack_stats,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::super::profile::decode_player_profile;
    use super::*;
    use leocard_minecraft_guess::{GuessCategoryCount, GuessCategoryStatistics};
    use leocard_protocol::{MatchId, MinecraftGuessProfileStats};

    #[test]
    fn previous_archive_preserves_identity_ratings_and_all_existing_game_statistics() {
        let previous = V2PlayerProfile {
            secret_key: [23; 32],
            games: V2GameProfiles {
                qigui523: StoredRatingProfile {
                    reference_points: 120,
                    completed_games: 11,
                    applied_matches: vec![MatchId([7; 16])],
                    qigui523_stats: None,
                },
                texas_holdem_stats: Some(V3TexasHoldemStats {
                    completed_games: 4,
                    ..Default::default()
                }),
                shengji_stats: Some(ShengjiProfileStats::default()),
                uno_stats: Some(UnoProfileStats::default()),
                interaction_stats: Some(PlayerInteractionStats::default()),
                mahjong_stats: Some(MahjongProfileStats::default()),
                minecraft_guess_stats: Some(PreviousMinecraftGuessStats {
                    completed_games: 9,
                    total_reference_delta: -3,
                    total_score: 83,
                    placement_counts: [1, 2, 0, 1, 0, 2, 2, 1],
                    block_games: 20,
                    block_wins: 4,
                    item_games: 12,
                    item_wins: 3,
                    known_guesses: 70,
                    non_worse_guesses: 40,
                    unknown_guesses: 8,
                    unknown_enabled_guesses: 28,
                    skips: 9,
                }),
                blackjack_stats: Some(BlackjackProfileStats {
                    completed_games: 5,
                    ..Default::default()
                }),
            },
        };
        let expected = MinecraftGuessProfileStats {
            completed_games: 9,
            total_reference_delta: -3,
            total_score: 83,
            placement_counts: [1, 2, 0, 1, 0, 2, 2, 1],
            categories: GuessCategoryStatistics {
                block: GuessCategoryCount { games: 20, wins: 4 },
                item: GuessCategoryCount { games: 12, wins: 3 },
                ..Default::default()
            },
            known_guesses: 70,
            non_worse_guesses: 40,
            unknown_guesses: 8,
            unknown_enabled_guesses: 28,
            skips: 9,
        };
        let mut bytes = V2_PROFILE_HEADER.to_vec();
        bytes.extend(postcard::to_allocvec(&previous).unwrap());
        let migrated = decode_player_profile(&bytes).unwrap();
        assert_eq!(migrated.secret_key, [23; 32]);
        assert_eq!(migrated.games.qigui523.reference_points, 120);
        assert_eq!(
            migrated.games.qigui523.applied_matches,
            vec![MatchId([7; 16])]
        );
        assert_eq!(
            migrated.games.minecraft_guess_stats.as_ref(),
            Some(&expected)
        );
        assert_eq!(expected.categories.total().games, 32);
        assert_eq!(expected.categories.mob.games, 0);
        assert_eq!(expected.wins(), 7);
        assert_eq!(
            (
                expected.known_guesses,
                expected.non_worse_guesses,
                expected.skips
            ),
            (70, 40, 9)
        );
        assert_eq!(
            (expected.unknown_guesses, expected.unknown_enabled_guesses),
            (8, 28)
        );
        assert_eq!(
            migrated.games.texas_holdem_stats,
            previous.games.texas_holdem_stats.map(Into::into)
        );
        assert_eq!(migrated.games.shengji_stats, previous.games.shengji_stats);
        assert_eq!(migrated.games.uno_stats, previous.games.uno_stats);
        assert_eq!(
            migrated.games.interaction_stats,
            previous.games.interaction_stats
        );
        assert_eq!(migrated.games.mahjong_stats, previous.games.mahjong_stats);
        assert_eq!(
            migrated.games.blackjack_stats,
            previous.games.blackjack_stats
        );
        let encoded = migrated.encode().unwrap();
        let reloaded = decode_player_profile(&encoded).unwrap();
        assert_eq!(reloaded.games.minecraft_guess_stats, Some(expected));
    }
}
