use super::super::migration::legacy;
use super::*;
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
            games
                .qigui523
                .clone()
                .map(legacy::QiGui523ProfileStats::from),
            games.texas.clone().map(legacy::TexasProfileStats::from),
            games.shengji.clone().map(legacy::ShengjiProfileStats::from),
            games.uno.clone().map(legacy::UnoProfileStats::from),
            games.mahjong.clone().map(legacy::MahjongProfileStats::from),
            games
                .mc_guess
                .clone()
                .map(legacy::McGuessProfileStats::from),
            games
                .blackjack
                .clone()
                .map(legacy::BlackjackProfileStats::from),
            games
                .doudizhu
                .clone()
                .map(legacy::DoudizhuProfileStats::from),
            games.guandan.clone().map(legacy::GuandanProfileStats::from),
        ),
    ))
    .unwrap();
    let mut old = b"LeoCard/profile/v8\0".to_vec();
    old.extend_from_slice(&payload);
    let directory = temp_dir().join(format!(
        "leocard-profile-migration-v8-{:016x}",
        fastrand::u64(..)
    ));
    fs::create_dir(&directory).unwrap();
    let path = directory.join("profile.dat");
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
        assert!(
            fs::read(&path)
                .unwrap()
                .starts_with(b"LeoCard/profile/v9\0")
        );
    }
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn game_profile_timing_survives_save_and_reload() {
    let directory = temp_dir().join(format!("leocard-profile-v9-{:016x}", fastrand::u64(..)));
    fs::create_dir(&directory).unwrap();
    let path = directory.join("profile.dat");
    let mut profile = LocalPlayerProfile::load_or_create_at(&path).unwrap();
    let key = profile.identity.secret_bytes();
    let mut games = complete_game_profiles();
    let timing = &mut games.guandan.as_mut().unwrap().action_time;
    timing.record(60_250);
    timing.record(750);
    assert!(profile.sync_game_profiles(&games));
    profile.store().write(&path).unwrap();

    let restored = LocalPlayerProfile::load_or_create_at(&path).unwrap();
    assert_eq!(restored.identity.secret_bytes(), key);
    assert_eq!(restored.game_profiles(), &games);
    fs::remove_dir_all(directory).unwrap();
}
