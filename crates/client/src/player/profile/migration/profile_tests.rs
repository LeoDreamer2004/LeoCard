use super::*;
use leocard_protocol::{MinecraftGuessProfileStats, QiGui523ProfileStats};

#[test]
fn old_profile_keeps_identity_ratings_all_game_records_and_receipts() {
    let previous = V080StoredPlayerProfile {
        secret_key: [17; 32],
        games: V080StoredGameProfiles {
            qigui523: StoredRatingProfile {
                reference_points: 120,
                completed_games: 27,
                applied_matches: vec![MatchId([8; 16])],
                qigui523_stats: Some(QiGui523ProfileStats {
                    completed_games: 4,
                    total_score: 230,
                    ..Default::default()
                }),
            },
            texas_holdem_stats: Some(TexasHoldemProfileStats {
                completed_games: 3,
                total_reference_delta: -2,
                ..Default::default()
            }),
            shengji_stats: Some(ShengjiProfileStats {
                completed_games: 2,
                plays: 35,
                ..Default::default()
            }),
            uno_stats: Some(SixSeatUnoStats {
                completed_games: 1,
                uno_calls: 8,
                ..Default::default()
            }),
            interaction_stats: Some(PlayerInteractionStats {
                flowers_received: 45,
                eggs_received: 6,
            }),
            mahjong_stats: Some(MahjongProfileStats {
                completed_games: 7,
                wins: 12,
                ..Default::default()
            }),
        },
    };
    let old = postcard::to_allocvec(&previous).unwrap();
    let mut migrated = decode_player_profile(&old).unwrap();
    assert!(migrated.games.minecraft_guess_stats.is_none());
    assert_eq!(migrated.secret_key, previous.secret_key);
    assert_eq!(
        postcard::to_allocvec(&migrated.games.qigui523).unwrap(),
        postcard::to_allocvec(&previous.games.qigui523).unwrap()
    );
    assert_eq!(
        migrated.games.texas_holdem_stats,
        previous.games.texas_holdem_stats
    );
    assert_eq!(migrated.games.shengji_stats, previous.games.shengji_stats);
    assert_eq!(
        migrated.games.uno_stats,
        previous.games.uno_stats.map(Into::into)
    );
    assert_eq!(
        migrated.games.interaction_stats,
        previous.games.interaction_stats
    );
    assert_eq!(migrated.games.mahjong_stats, previous.games.mahjong_stats);
    migrated.games.minecraft_guess_stats = Some(MinecraftGuessProfileStats {
        completed_games: 1,
        block_games: 6,
        block_wins: 3,
        ..Default::default()
    });
    let upgraded = migrated.encode().unwrap();
    let reloaded = decode_player_profile(&upgraded).unwrap();
    assert_eq!(
        reloaded.games.minecraft_guess_stats,
        migrated.games.minecraft_guess_stats
    );
    assert!(decode_player_profile(&upgraded[..upgraded.len() - 1]).is_err());
}

#[test]
fn six_seat_guess_archive_keeps_statistics_and_adds_empty_seventh_and_eighth_places() {
    let previous = SixSeatStoredPlayerProfile {
        secret_key: [23; 32],
        games: SixSeatStoredGameProfiles {
            qigui523: StoredRatingProfile {
                reference_points: 12,
                completed_games: 8,
                applied_matches: vec![MatchId([9; 16])],
                qigui523_stats: None,
            },
            texas_holdem_stats: None,
            shengji_stats: None,
            uno_stats: None,
            interaction_stats: None,
            mahjong_stats: None,
            minecraft_guess_stats: Some(SixSeatMinecraftGuessStats {
                completed_games: 8,
                total_reference_delta: -3,
                total_score: 152,
                placement_counts: [1, 1, 2, 1, 1, 2],
                block_games: 20,
                block_wins: 3,
                item_games: 12,
                item_wins: 2,
                known_guesses: 35,
                non_worse_guesses: 19,
                unknown_guesses: 4,
                unknown_enabled_guesses: 12,
                skips: 6,
            }),
        },
    };
    let bytes = postcard::to_allocvec(&previous).unwrap();
    let migrated = decode_player_profile(&bytes).unwrap();
    assert_eq!(migrated.secret_key, previous.secret_key);
    assert_eq!(migrated.games.qigui523.reference_points, 12);
    let stats = migrated.games.minecraft_guess_stats.as_ref().unwrap();
    assert_eq!(stats.placement_counts, [1, 1, 2, 1, 1, 2, 0, 0]);
    assert_eq!(
        (stats.small_games(), stats.wins(), stats.actions()),
        (32, 5, 45)
    );
    assert_eq!(stats.total_score, 152);
    let current = migrated.encode().unwrap();
    assert_eq!(
        decode_player_profile(&current)
            .unwrap()
            .games
            .minecraft_guess_stats,
        migrated.games.minecraft_guess_stats
    );
}

#[test]
fn six_seat_uno_archive_keeps_current_guess_statistics_and_extends_uno_placements() {
    let previous = SixSeatUnoStoredPlayerProfile {
        secret_key: [31; 32],
        games: SixSeatUnoStoredGameProfiles {
            qigui523: StoredRatingProfile {
                reference_points: 120,
                completed_games: 8,
                applied_matches: vec![MatchId([9; 16])],
                qigui523_stats: None,
            },
            texas_holdem_stats: None,
            shengji_stats: None,
            uno_stats: Some(SixSeatUnoStats {
                completed_games: 8,
                total_reference_delta: -4,
                total_remaining_score: 213,
                placement_counts: [1, 1, 2, 1, 1, 2],
                max_penalty_cards: 50,
                uno_calls: 23,
                ..Default::default()
            }),
            interaction_stats: None,
            mahjong_stats: None,
            minecraft_guess_stats: Some(MinecraftGuessProfileStats {
                completed_games: 8,
                placement_counts: [1; 8],
                block_wins: 3,
                ..Default::default()
            }),
        },
    };
    let bytes = postcard::to_allocvec(&previous).unwrap();
    let mut migrated = decode_player_profile(&bytes).unwrap();
    assert_eq!(migrated.secret_key, previous.secret_key);
    assert_eq!(
        migrated.games.qigui523.applied_matches,
        previous.games.qigui523.applied_matches
    );
    assert_eq!(
        migrated.games.minecraft_guess_stats,
        previous.games.minecraft_guess_stats
    );
    let stats = migrated.games.uno_stats.as_mut().unwrap();
    assert_eq!(stats.placement_counts, [1, 1, 2, 1, 1, 2, 0, 0]);
    assert_eq!(
        (
            stats.completed_games,
            stats.total_reference_delta,
            stats.total_remaining_score
        ),
        (8, -4, 213)
    );
    assert_eq!((stats.max_penalty_cards, stats.uno_calls), (50, 23));
    stats.placement_counts[7] = 2;
    let current = migrated.encode().unwrap();
    assert_eq!(
        decode_player_profile(&current).unwrap().games.uno_stats,
        migrated.games.uno_stats
    );
    assert!(decode_player_profile(&current[..current.len() - 1]).is_err());
}
