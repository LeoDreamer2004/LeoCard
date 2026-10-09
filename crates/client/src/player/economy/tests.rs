use super::{
    migration,
    rewards::login_day,
    storage::{EconomyArchive, FORMAT_VERSION},
};
use leocard_achievements::{ACHIEVEMENT_REGISTRY, AchievementTier};
use leocard_protocol::{MatchId, PlayerId, PlayerReferenceChange, ProfileId};

fn changes(deltas: &[i16]) -> Vec<PlayerReferenceChange> {
    deltas
        .iter()
        .enumerate()
        .map(|(index, delta)| PlayerReferenceChange {
            player: PlayerId(index as u8),
            profile_id: ProfileId([index as u8 + 1; 32]),
            delta: *delta,
        })
        .collect()
}

#[test]
fn settlement_uses_platform_points_and_the_shared_rounded_down_ticket() {
    let changes = changes(&[5, 0, -1, -4]);
    for (index, expected) in [123, 98, 93, 78].into_iter().enumerate() {
        let mut wallet = EconomyArchive::new(changes[index].profile_id);
        wallet.coins = 100;
        assert!(wallet.settle_match(MatchId([1; 16]), &changes));
        assert_eq!(wallet.coins, expected);
        assert!(!wallet.settle_match(MatchId([1; 16]), &changes));
        assert_eq!(wallet.coins, expected);
    }
}

#[test]
fn balances_never_go_below_zero_or_overflow() {
    let changes = changes(&[i16::MIN, i16::MAX]);
    let mut wallet = EconomyArchive::new(changes[0].profile_id);
    wallet.coins = 3;
    assert!(wallet.settle_match(MatchId([2; 16]), &changes));
    assert_eq!(wallet.coins, 0);
    wallet.change_coins(i64::from(u32::MAX));
    wallet.change_coins(1500);
    assert_eq!(wallet.coins, u32::MAX);
    assert!(!wallet.settle_match(MatchId([3; 16]), &changes[1..]));
    assert!(!wallet.settled_matches.contains(&MatchId([3; 16])));
}

#[test]
fn daily_login_resets_at_beijing_midnight_and_does_not_replay_on_clock_rollback() {
    assert_eq!(login_day(16 * 3600 - 1), 0);
    assert_eq!(login_day(16 * 3600), 1);
    let mut wallet = EconomyArchive::new(ProfileId([1; 32]));
    assert!(wallet.reward_daily(1));
    assert!(!wallet.reward_daily(1));
    assert!(!wallet.reward_daily(0));
    assert!(wallet.reward_daily(2));
    assert_eq!(wallet.coins, 200);
}

#[test]
fn achievement_tiers_pay_once_each() {
    let mut wallet = EconomyArchive::new(ProfileId([1; 32]));
    for (tier, total) in [
        (AchievementTier::Bronze, 50),
        (AchievementTier::Silver, 450),
        (AchievementTier::Gold, 1950),
    ] {
        let definition = ACHIEVEMENT_REGISTRY
            .iter()
            .find(|definition| definition.tier == tier)
            .unwrap();
        assert!(wallet.reward_achievement(definition));
        assert!(!wallet.reward_achievement(definition));
        assert_eq!(wallet.coins, total);
    }
}

#[test]
fn reloading_preserves_reward_and_settlement_receipts() {
    let changes = changes(&[5, -5]);
    let mut wallet = EconomyArchive::new(changes[0].profile_id);
    let definition = &ACHIEVEMENT_REGISTRY[0];
    wallet.reward_daily(12);
    wallet.reward_achievement(definition);
    wallet.settle_match(MatchId([4; 16]), &changes);
    let coins = wallet.coins;
    let mut reloaded = migration::decode(
        FORMAT_VERSION,
        &postcard::to_allocvec(&wallet).unwrap(),
        wallet.profile_id,
    )
    .unwrap();
    assert!(!reloaded.reward_daily(12));
    assert!(!reloaded.reward_achievement(definition));
    assert!(!reloaded.settle_match(MatchId([4; 16]), &changes));
    assert_eq!(reloaded.coins, coins);
}

#[test]
fn old_wallets_restart_and_unknown_or_wrong_identity_archives_are_rejected() {
    let profile_id = ProfileId([1; 32]);
    for version in [1, 2] {
        let wallet = migration::decode(version, &[], profile_id).unwrap();
        assert_eq!(wallet.coins, 0);
        assert!(wallet.active_items.is_empty());
        assert_eq!(wallet.last_login_day, None);
    }
    assert!(migration::decode(FORMAT_VERSION + 1, &[], profile_id).is_err());
    let wallet = EconomyArchive::new(ProfileId([2; 32]));
    assert!(
        migration::decode(
            FORMAT_VERSION,
            &postcard::to_allocvec(&wallet).unwrap(),
            profile_id
        )
        .is_err()
    );
}
