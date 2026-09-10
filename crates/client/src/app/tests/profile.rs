use super::prelude::*;
use leocard_client::PlayerIdentity;
use leocard_protocol::{MatchId, PlayerGameProfiles, PlayerId, PlayerReferenceChange};
use std::collections::HashSet;

#[test]
fn local_profile_applies_each_finished_match_once() {
    let identity = PlayerIdentity::from_secret_bytes([7; 32]);
    let profile_id = identity.profile_id();
    let mut profile = LocalPlayerProfile {
        identity,
        rating: PlayerRatingProfile {
            reference_points: 10,
            completed_games: 4,
            applied_matches: HashSet::new(),
            last_change: None,
        },
        game_profiles: PlayerGameProfiles::default(),
    };
    let match_id = MatchId([3; 16]);
    let changes = [PlayerReferenceChange {
        player: PlayerId(0),
        profile_id,
        delta: 3,
    }];

    assert!(profile.apply_finished_match(match_id, &changes));
    assert!(!profile.apply_finished_match(match_id, &changes));
    assert_eq!(profile.rating.reference_points, 13);
    assert_eq!(profile.rating.completed_games, 5);
    assert_eq!(profile.rating.last_change, Some((match_id, 3)));
}

#[test]
fn reference_levels_use_the_declared_boundaries() {
    assert_eq!(reference_level(1_001), "下界合金");
    assert_eq!(reference_level(1_000), "钻石");
    assert_eq!(reference_level(500), "钻石");
    assert_eq!(reference_level(499), "金");
    assert_eq!(reference_level(200), "金");
    assert_eq!(reference_level(199), "红石");
    assert_eq!(reference_level(100), "红石");
    assert_eq!(reference_level(99), "铁");
    assert_eq!(reference_level(50), "铁");
    assert_eq!(reference_level(49), "铜");
    assert_eq!(reference_level(10), "铜");
    assert_eq!(reference_level(9), "圆石");
    assert_eq!(reference_level(0), "圆石");
    assert_eq!(reference_level(-1), "木头");
    assert_eq!(reference_level(-10), "木头");
    assert_eq!(reference_level(-11), "泥土");
    assert_eq!(reference_level(-50), "泥土");
    assert_eq!(reference_level(-51), "堆肥桶");
    assert_eq!(reference_points_label(500), "等级:钻石  分数:500");
}
