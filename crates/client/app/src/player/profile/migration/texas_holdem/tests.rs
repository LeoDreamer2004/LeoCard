use super::super::super::StoredRatingProfile;
use super::super::decode_player_profile;
use super::archive::*;
use leocard_protocol::{
    BlackjackProfileStats, MahjongProfileStats, MatchId, MinecraftGuessProfileStats,
    PlayerInteractionStats, QiGui523ProfileStats, ShengjiProfileStats, TexasHoldemProfileStats,
    UnoProfileStats,
};

#[test]
fn six_seat_archive_migrates_and_round_trips_without_losing_player_data() {
    let key = [23u8; 32];
    let rating = StoredRatingProfile {
        reference_points: 123,
        completed_games: 21,
        applied_matches: vec![MatchId([7; 16])],
        qigui523_stats: Some(QiGui523ProfileStats::default()),
    };
    let statistics = V3TexasHoldemStats {
        completed_games: 21,
        total_reference_delta: -17,
        total_final_chips: 613,
        placement_counts: [1, 2, 3, 4, 5, 6],
        wagered_chips: 900,
        wager_actions: 47,
        voluntary_actions: 64,
        check_actions: 11,
        raise_actions: 13,
        all_in_actions: 7,
        hands_played: 35,
        hands_folded: 12,
        hand_category_counts: [4; 10],
    };
    // Encode the old flat struct field by field, independently of the decoder's
    // nested representation. Postcard structs carry no framing of their own.
    let mut bytes = V4_PROFILE_HEADER.to_vec();
    bytes.extend(postcard::to_allocvec(&key).unwrap());
    bytes.extend(postcard::to_allocvec(&rating).unwrap());
    bytes.push(1); // Some(TexasHoldemProfileStats)
    bytes.extend(postcard::to_allocvec(&statistics).unwrap());
    bytes.extend(postcard::to_allocvec(&(35u32, 19u32)).unwrap());
    let uno = Some(UnoProfileStats::default());
    let mahjong = Some(MahjongProfileStats::default());
    bytes.extend(
        postcard::to_allocvec(&(
            None::<ShengjiProfileStats>,
            &uno,
            None::<PlayerInteractionStats>,
            &mahjong,
            None::<MinecraftGuessProfileStats>,
            None::<BlackjackProfileStats>,
        ))
        .unwrap(),
    );
    let migrated = decode_player_profile(&bytes).unwrap();
    assert_eq!(migrated.secret_key, key);
    assert_eq!(migrated.games.qigui523.reference_points, 123);
    assert_eq!(
        migrated.games.qigui523.applied_matches,
        rating.applied_matches
    );
    assert_eq!(migrated.games.uno_stats, uno);
    assert_eq!(migrated.games.mahjong_stats, mahjong);
    let expected = TexasHoldemProfileStats {
        entry_tracked_hands: 35,
        entered_hands: 19,
        ..statistics.into()
    };
    assert_eq!(migrated.games.texas_holdem_stats.as_ref(), Some(&expected));
    assert_eq!(expected.placement_counts, [1, 2, 3, 4, 5, 6, 0, 0]);
    let saved = migrated.encode().unwrap();
    assert!(saved.starts_with(super::super::super::PROFILE_HEADER));
    let reloaded = decode_player_profile(&saved).unwrap();
    assert_eq!(reloaded.games.texas_holdem_stats, Some(expected));
}
