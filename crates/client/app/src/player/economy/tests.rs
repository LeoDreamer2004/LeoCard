mod consumables;

use super::rewards::login_day;
use super::storage::{EconomyArchive, FORMAT_VERSION};
use super::{PropId, ShopAvatarFrame, migration};
use leocard_achievements::AchievementTier;
use leocard_game_common::PlayerId;
use leocard_protocol::{
    ACHIEVEMENT_REGISTRY, AvatarFrameId, MatchId, PlayerReferenceChange, ProfileId,
};

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
        wallet.ledger.coins = 100;
        assert!(wallet.ledger.settle_match(MatchId([1; 16]), &changes));
        assert_eq!(wallet.ledger.coins, expected);
        assert!(!wallet.ledger.settle_match(MatchId([1; 16]), &changes));
        assert_eq!(wallet.ledger.coins, expected);
    }
}

#[test]
fn balances_never_go_below_zero_or_overflow() {
    let changes = changes(&[i16::MIN, i16::MAX]);
    let mut wallet = EconomyArchive::new(changes[0].profile_id);
    wallet.ledger.coins = 3;
    assert!(wallet.ledger.settle_match(MatchId([2; 16]), &changes));
    assert_eq!(wallet.ledger.coins, 0);
    wallet.ledger.change_coins(i64::from(u32::MAX));
    wallet.ledger.change_coins(1500);
    assert_eq!(wallet.ledger.coins, u32::MAX);
    assert!(!wallet.ledger.settle_match(MatchId([3; 16]), &changes[1..]));
    assert!(!wallet.ledger.settled_matches.contains(&MatchId([3; 16])));
}

#[test]
fn daily_login_resets_at_beijing_midnight_and_does_not_replay_on_clock_rollback() {
    assert_eq!(login_day(16 * 3600 - 1), 0);
    assert_eq!(login_day(16 * 3600), 1);
    let mut wallet = EconomyArchive::new(ProfileId([1; 32]));
    assert!(wallet.ledger.reward_daily(1));
    assert!(!wallet.ledger.reward_daily(1));
    assert!(!wallet.ledger.reward_daily(0));
    assert!(wallet.ledger.reward_daily(2));
    assert_eq!(wallet.ledger.coins, 200);
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
        assert!(wallet.ledger.reward_achievement(definition));
        assert!(!wallet.ledger.reward_achievement(definition));
        assert_eq!(wallet.ledger.coins, total);
    }
}

#[test]
fn reloading_preserves_reward_and_settlement_receipts() {
    let changes = changes(&[5, -5]);
    let mut wallet = EconomyArchive::new(changes[0].profile_id);
    let definition = &ACHIEVEMENT_REGISTRY[0];
    wallet.ledger.reward_daily(12);
    wallet.ledger.reward_achievement(definition);
    wallet.ledger.settle_match(MatchId([4; 16]), &changes);
    let coins = wallet.ledger.coins;
    let mut reloaded = migration::decode(
        FORMAT_VERSION,
        &postcard::to_allocvec(&wallet).unwrap(),
        wallet.ledger.profile_id,
        None,
        0,
    )
    .unwrap();
    assert!(!reloaded.ledger.reward_daily(12));
    assert!(!reloaded.ledger.reward_achievement(definition));
    assert!(!reloaded.ledger.settle_match(MatchId([4; 16]), &changes));
    assert_eq!(reloaded.ledger.coins, coins);
}

#[test]
fn adding_avatar_frames_preserves_the_v3_wallet_ledger() {
    let profile_id = ProfileId([1; 32]);
    let mut old = EconomyArchive::new(profile_id).ledger;
    old.reward_daily(12);
    old.reward_achievement(&ACHIEVEMENT_REGISTRY[0]);
    old.settled_matches.insert(MatchId([7; 16]));
    old.active_props.insert(PropId::ObservationLens, 12345);
    old.rewards_initialized = true;
    old.coins = 5233;
    let payload = postcard::to_allocvec(&old).unwrap();
    let migrated =
        migration::decode(3, &payload, profile_id, Some(AvatarFrameId::Host), 0).unwrap();
    assert_eq!(postcard::to_allocvec(&migrated.ledger).unwrap(), payload);
    assert!(migrated.owns_frame(AvatarFrameId::Default));
    assert!(migrated.owned_frames.is_empty());
    assert_eq!(migrated.equipped_frame, AvatarFrameId::Host);
    assert!(migrated.can_use_frame(AvatarFrameId::Host));
    assert!(!migrated.owns_frame(AvatarFrameId::Host));
    assert!(migration::decode(3, &payload, ProfileId([2; 32]), None, 0).is_err());
}

#[test]
fn upgrading_a_wallet_with_a_frame_preserves_the_equipped_choice() {
    let profile_id = ProfileId([1; 32]);
    let mut old = EconomyArchive::new(profile_id);
    old.owned_frames.insert(AvatarFrameId::Study);
    let payload =
        postcard::to_allocvec(&(&old.ledger, &old.owned_frames, AvatarFrameId::Study)).unwrap();
    let mut migrated =
        migration::decode(4, &payload, profile_id, Some(AvatarFrameId::Cattail), 0).unwrap();
    assert!(!migrated.update_exclusive_frame(Some(AvatarFrameId::Cattail)));
    assert_eq!(migrated.equipped_frame, AvatarFrameId::Study);
    assert!(migrated.can_use_frame(AvatarFrameId::Cattail));
    assert_eq!(migrated.owned_frames, old.owned_frames);
}

#[test]
fn persona_changes_equip_the_new_frame_and_only_keep_departed_frames_if_owned() {
    use AvatarFrameId::{Cattail, Default, Horse, Host, IronBasin, Orange};

    let mut wallet = EconomyArchive::new(ProfileId([1; 32]));
    for (name, frame) in [
        ("Yang120", Orange),
        ("DreamerLeo", Cattail),
        ("wpy03", IronBasin),
        ("SnowyWind", Horse),
        ("Cnofne", Host),
    ] {
        let exclusive = ShopAvatarFrame::exclusive_for_name(name);
        assert_eq!(exclusive, Some(frame));
        assert!(wallet.update_exclusive_frame(exclusive));
        assert_eq!(wallet.equipped_frame, frame);
        assert!(wallet.can_use_frame(frame));
        assert!(!wallet.owns_frame(frame));
    }
    let regular = ShopAvatarFrame::exclusive_for_name("玩家");
    assert_eq!(regular, None);
    assert!(wallet.update_exclusive_frame(regular));
    assert_eq!(wallet.equipped_frame, Default);
    assert!(!wallet.can_use_frame(Host));

    wallet.owned_frames.insert(Host);
    assert!(wallet.update_exclusive_frame(Some(Host)));
    assert!(wallet.update_exclusive_frame(regular));
    assert_eq!(wallet.equipped_frame, Host);
}

#[test]
fn old_wallets_restart_and_unknown_or_wrong_identity_archives_are_rejected() {
    let profile_id = ProfileId([1; 32]);
    for version in [1, 2] {
        let wallet = migration::decode(version, &[], profile_id, None, 0).unwrap();
        assert_eq!(wallet.ledger.coins, 0);
        assert!(wallet.ledger.active_props.is_empty());
        assert_eq!(wallet.ledger.last_login_day, None);
    }
    assert!(migration::decode(FORMAT_VERSION + 1, &[], profile_id, None, 0).is_err());
    let wallet = EconomyArchive::new(ProfileId([2; 32]));
    assert!(
        migration::decode(
            FORMAT_VERSION,
            &postcard::to_allocvec(&wallet).unwrap(),
            profile_id,
            None,
            0,
        )
        .is_err()
    );
}

#[test]
fn old_card_counters_merge_paid_time_into_one_shared_entitlement() {
    let profile_id = ProfileId([1; 32]);
    let now = 10_000;
    let mut old = EconomyArchive::new(profile_id);
    old.ledger
        .active_props
        .insert(PropId::ShengjiCardCounter, now + 1_200);
    old.ledger
        .active_props
        .insert(PropId::QiGui523CardCounter, now - 1);
    old.ledger
        .active_props
        .insert(PropId::DoudizhuCardCounter, now + 1_800);
    old.ledger
        .active_props
        .insert(PropId::ObservationLens, now + 500);
    let payload = postcard::to_allocvec(&(
        &old.ledger,
        &old.owned_frames,
        old.equipped_frame,
        old.exclusive_frame,
    ))
    .unwrap();
    let migrated = migration::decode(5, &payload, profile_id, None, now).unwrap();
    assert_eq!(
        migrated.ledger.active_props[&PropId::CardCounter],
        now + 3_000
    );
    assert_eq!(
        migrated.ledger.active_props[&PropId::ObservationLens],
        now + 500
    );
    for legacy in [
        PropId::ShengjiCardCounter,
        PropId::QiGui523CardCounter,
        PropId::DoudizhuCardCounter,
    ] {
        assert!(!migrated.ledger.active_props.contains_key(&legacy));
    }
    let reloaded = migration::decode(
        FORMAT_VERSION,
        &postcard::to_allocvec(&migrated).unwrap(),
        profile_id,
        None,
        now + 100,
    )
    .unwrap();
    assert_eq!(
        reloaded.ledger.active_props, migrated.ledger.active_props,
        "a current wallet must not add inherited time a second time"
    );
}

#[test]
fn merging_card_counters_saturates_at_the_latest_representable_expiry() {
    let profile_id = ProfileId([1; 32]);
    let now = u64::MAX - 100;
    let mut old = EconomyArchive::new(profile_id);
    old.ledger
        .active_props
        .insert(PropId::ShengjiCardCounter, u64::MAX);
    old.ledger
        .active_props
        .insert(PropId::DoudizhuCardCounter, u64::MAX);
    let migrated = migration::decode(
        5,
        &postcard::to_allocvec(&(
            &old.ledger,
            &old.owned_frames,
            old.equipped_frame,
            old.exclusive_frame,
        ))
        .unwrap(),
        profile_id,
        None,
        now,
    )
    .unwrap();
    assert_eq!(migrated.ledger.active_props[&PropId::CardCounter], u64::MAX);
}
