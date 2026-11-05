use super::*;
use leocard_protocol::GameKind;
use serde_json::Value;
use std::env::temp_dir;

fn complete_game_profiles() -> PlayerGameProfiles {
    PlayerGameProfiles {
        qigui523: Some(QiGui523ProfileStats {
            completed_games: 31,
            total_score: 840,
            ..Default::default()
        }),
        texas: Some(TexasProfileStats {
            completed_games: 32,
            entered_hands: 13,
            ..Default::default()
        }),
        shengji: Some(ShengjiProfileStats {
            completed_games: 33,
            buried_points: 80,
            ..Default::default()
        }),
        uno: Some(UnoProfileStats {
            completed_games: 34,
            successful_jump_ins: 8,
            ..Default::default()
        }),
        mahjong: Some(MahjongProfileStats {
            completed_games: 35,
            total_win_fan: 128,
            ..Default::default()
        }),
        mc_guess: Some(McGuessProfileStats {
            completed_games: 36,
            known_guesses: 45,
            ..Default::default()
        }),
        blackjack: Some(BlackjackProfileStats {
            completed_games: 37,
            total_chips_delta: -105,
            ..Default::default()
        }),
        doudizhu: Some(DoudizhuProfileStats {
            completed_games: 38,
            bomb_plays: 3,
            ..Default::default()
        }),
        guandan: Some(GuandanProfileStats {
            completed_games: 39,
            straight_flush_plays: 7,
            ..Default::default()
        }),
        interactions: Some(PlayerInteractionStats {
            flowers_received: 99,
            eggs_received: 21,
        }),
        ..Default::default()
    }
}

#[test]
fn loading_v8_rewrites_to_v9_without_losing_any_profile_data() {
    let key = [73; 32];
    let receipts = vec![MatchId([19; 16]), MatchId([23; 16])];
    let rating = (-123i32, 47u32, &receipts);
    let games = complete_game_profiles();
    // Freeze the upstream v8 layout independently of the timing-enabled DTO.
    let payload = postcard::to_allocvec(&(
        key,
        rating,
        &games.interactions,
        (
            &games.qigui523,
            &games.texas,
            &games.shengji,
            &games.uno,
            &games.mahjong,
            &games.mc_guess,
            &games.blackjack,
            &games.doudizhu,
            &games.guandan,
        ),
    ))
    .unwrap();
    let mut old = b"LeoCard/profile/v8\0".to_vec();
    old.extend_from_slice(&payload);
    let mut expected = b"LeoCard/profile/v9\0".to_vec();
    expected.extend_from_slice(&payload);
    expected.extend(postcard::to_allocvec(&GameActionTimes::default()).unwrap());
    assert_eq!(PROFILE_HEADER, b"LeoCard/profile/v9\0");

    let directory = temp_dir().join(format!(
        "leocard-profile-migration-v8-{:016x}",
        fastrand::u64(..)
    ));
    fs::create_dir(&directory).unwrap();
    let path = directory.join("profile.dat");
    let mut trailing = old.clone();
    trailing.push(0);
    for damaged in [&old[..old.len() - 1], trailing.as_slice()] {
        fs::write(&path, damaged).unwrap();
        assert!(LocalPlayerProfile::load_or_create_at(&path).is_err());
        assert_eq!(fs::read(&path).unwrap(), damaged);
    }

    fs::write(&path, old).unwrap();
    for _ in 0..2 {
        let profile = LocalPlayerProfile::load_or_create_at(&path).unwrap();
        assert_eq!(profile.identity.secret_bytes(), key);
        assert_eq!(profile.reference_points(), rating.0);
        assert_eq!(profile.completed_games(), rating.1);
        assert_eq!(
            profile.rating.applied_matches,
            receipts.iter().copied().collect()
        );
        assert_eq!(profile.game_profiles(), &games);
        assert!(profile.game_profiles().action_times == GameActionTimes::default());
        assert_eq!(fs::read(&path).unwrap(), expected);
    }
    assert_eq!(fs::read_dir(&directory).unwrap().count(), 1);
    fs::remove_dir_all(directory).unwrap();
}

fn populated_action_times() -> GameActionTimes {
    // Build nonzero fixtures through serde, without depending on record or
    // the container's public fields. Each game's counters remain distinct.
    fn populate(value: &mut Value, count: &mut u64) {
        match value {
            Value::Object(fields)
                if fields.contains_key("total_millis")
                    && fields.contains_key("completed_actions") =>
            {
                *count += 1;
                fields.insert("total_millis".into(), Value::from(12_345_678_900 + *count));
                fields.insert("longest_millis".into(), Value::from(6_000_000_000 + *count));
                fields.insert(
                    "completed_actions".into(),
                    Value::from(4_294_967_300 + *count),
                );
            }
            Value::Object(fields) => {
                for value in fields.values_mut() {
                    populate(value, count);
                }
            }
            Value::Array(values) => {
                for value in values {
                    populate(value, count);
                }
            }
            _ => {}
        }
    }

    let mut value = serde_json::to_value(GameActionTimes::default()).unwrap();
    let mut count = 0;
    populate(&mut value, &mut count);
    assert_eq!(count, 10);
    serde_json::from_value(value).unwrap()
}

#[test]
fn synced_action_times_and_maxima_survive_v9_save_and_reload() {
    let directory = temp_dir().join(format!("leocard-profile-v9-{:016x}", fastrand::u64(..)));
    fs::create_dir(&directory).unwrap();
    let path = directory.join("profile.dat");
    let mut profile = LocalPlayerProfile::load_or_create_at(&path).unwrap();
    let key = profile.identity.secret_bytes();
    let mut games = complete_game_profiles();
    assert!(profile.sync_game_profiles(&games));
    games.action_times = populated_action_times();
    assert!(profile.sync_game_profiles(&games));
    assert!(!profile.sync_game_profiles(&games));
    profile.store().write(&path).unwrap();

    let bytes = fs::read(&path).unwrap();
    assert!(bytes.starts_with(b"LeoCard/profile/v9\0"));
    let stored = migration::decode_player_profile(&bytes).unwrap();
    assert!(stored.action_times == games.action_times);
    let restored = LocalPlayerProfile::load_or_create_at(&path).unwrap();
    assert_eq!(restored.identity.secret_bytes(), key);
    assert_eq!(restored.game_profiles(), &games);
    for kind in [
        GameKind::QiGui523,
        GameKind::Texas,
        GameKind::Shengji,
        GameKind::Uno,
        GameKind::Mahjong,
        GameKind::McGuess,
        GameKind::Blackjack,
        GameKind::Doudizhu,
        GameKind::Guandan,
        GameKind::MahjongLink,
    ] {
        let expected = games.action_times.get(kind);
        let actual = restored.game_profiles().action_times.get(kind);
        assert!(expected.total_millis > u64::from(u32::MAX));
        assert!(expected.completed_actions > u64::from(u32::MAX));
        assert_eq!(actual.total_millis, expected.total_millis);
        assert_eq!(actual.completed_actions, expected.completed_actions);
        assert!(actual.longest_millis > u64::from(u32::MAX));
        assert_eq!(actual.longest_millis, expected.longest_millis);
    }
    assert_eq!(fs::read(&path).unwrap(), bytes);
    fs::remove_dir_all(directory).unwrap();
}
