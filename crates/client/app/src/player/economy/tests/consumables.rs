use super::super::storage::{EconomyArchive, FORMAT_VERSION};
use super::super::{PropId, migration};
use leocard_protocol::{AvatarFrameId, MatchId, ProfileId};

#[test]
fn egg_throwers_cost_one_hundred_each_and_can_be_consumed_exactly_once() {
    let prop = PropId::AutoEggThrower;
    assert_eq!(prop.definition().price, 100);
    assert!(prop.is_consumable());
    let mut wallet = EconomyArchive::new(ProfileId([1; 32]));
    wallet.ledger.coins = 200;
    wallet.buy_consumable(prop).unwrap();
    wallet.buy_consumable(prop).unwrap();
    assert_eq!(wallet.ledger.coins, 0);
    assert_eq!(wallet.consumables[&prop], 2);
    assert!(wallet.buy_consumable(prop).is_err());
    assert_eq!(wallet.consumables[&prop], 2);
    wallet.consume_prop(prop).unwrap();
    assert_eq!(wallet.consumables[&prop], 1);
    wallet.consume_prop(prop).unwrap();
    assert!(wallet.consumables.is_empty());
    assert!(wallet.consume_prop(prop).is_err());
    assert_eq!(wallet.ledger.coins, 0);
    assert!(wallet.ledger.active_props.is_empty());
}

#[test]
fn quantity_limits_and_wrong_item_types_do_not_spend_coins() {
    let mut wallet = EconomyArchive::new(ProfileId([1; 32]));
    wallet.ledger.coins = 200;
    wallet.consumables.insert(PropId::AutoEggThrower, u32::MAX);
    assert!(wallet.buy_consumable(PropId::AutoEggThrower).is_err());
    assert!(wallet.buy_consumable(PropId::ObservationLens).is_err());
    assert!(wallet.consume_prop(PropId::ObservationLens).is_err());
    assert_eq!(wallet.ledger.coins, 200);
    assert_eq!(wallet.consumables[&PropId::AutoEggThrower], u32::MAX);
}

#[test]
fn released_v6_migration_preserves_all_assets_and_the_shared_counter_expiry() {
    let profile = ProfileId([1; 32]);
    let mut old = EconomyArchive::new(profile);
    old.ledger.coins = 1234;
    for (index, prop) in [
        PropId::ObservationLens,
        PropId::ShengjiMissingSuitCard,
        PropId::UnoJumpInDevice,
        PropId::MahjongFanCalculator,
        PropId::CardCounter,
    ]
    .into_iter()
    .enumerate()
    {
        old.ledger.active_props.insert(prop, 98765 + index as u64);
    }
    old.ledger.last_login_day = Some(321);
    old.ledger.rewards_initialized = true;
    old.ledger
        .rewarded_achievements
        .insert("saved-reward".to_owned());
    old.ledger.settled_matches.insert(MatchId([2; 16]));
    old.owned_frames
        .extend([AvatarFrameId::Study, AvatarFrameId::Horse]);
    old.equipped_frame = AvatarFrameId::Study;
    old.exclusive_frame = Some(AvatarFrameId::Host);
    // Exactly the released upstream v6 layout, with no private-fork inventory.
    let payload = postcard::to_allocvec(&(
        &old.ledger,
        &old.owned_frames,
        old.equipped_frame,
        old.exclusive_frame,
    ))
    .unwrap();
    let migrated = migration::decode(6, &payload, profile, old.exclusive_frame, 50000).unwrap();
    assert_eq!(FORMAT_VERSION, 7);
    assert_eq!(
        postcard::to_allocvec(&migrated.ledger).unwrap(),
        postcard::to_allocvec(&old.ledger).unwrap()
    );
    assert_eq!(migrated.owned_frames, old.owned_frames);
    assert_eq!(migrated.equipped_frame, old.equipped_frame);
    assert_eq!(migrated.exclusive_frame, old.exclusive_frame);
    assert!(migrated.consumables.is_empty());
    let reloaded = migration::decode(
        FORMAT_VERSION,
        &postcard::to_allocvec(&migrated).unwrap(),
        profile,
        old.exclusive_frame,
        60000,
    )
    .unwrap();
    assert_eq!(reloaded.ledger.active_props, old.ledger.active_props);
    assert!(migration::decode(6, &payload, ProfileId([3; 32]), None, 0).is_err());
}

#[test]
fn older_wallet_paths_keep_assets_and_initialize_empty_inventory() {
    let profile = ProfileId([1; 32]);
    let mut old = EconomyArchive::new(profile);
    old.ledger.coins = 4321;
    old.ledger
        .active_props
        .insert(PropId::ShengjiCardCounter, 1300);
    old.ledger
        .active_props
        .insert(PropId::QiGui523CardCounter, 1400);
    old.ledger
        .active_props
        .insert(PropId::DoudizhuCardCounter, 900);
    old.ledger
        .active_props
        .insert(PropId::ObservationLens, 1800);
    old.owned_frames.insert(AvatarFrameId::Study);
    old.equipped_frame = AvatarFrameId::Study;
    old.exclusive_frame = Some(AvatarFrameId::Host);
    let payloads = [
        (3, postcard::to_allocvec(&old.ledger).unwrap()),
        (
            4,
            postcard::to_allocvec(&(&old.ledger, &old.owned_frames, old.equipped_frame)).unwrap(),
        ),
        (
            5,
            postcard::to_allocvec(&(
                &old.ledger,
                &old.owned_frames,
                old.equipped_frame,
                old.exclusive_frame,
            ))
            .unwrap(),
        ),
    ];
    for (version, payload) in payloads {
        let migrated =
            migration::decode(version, &payload, profile, old.exclusive_frame, 1000).unwrap();
        assert_eq!(migrated.ledger.coins, 4321);
        assert_eq!(migrated.ledger.active_props[&PropId::CardCounter], 1700);
        assert_eq!(migrated.ledger.active_props[&PropId::ObservationLens], 1800);
        assert_eq!(migrated.ledger.active_props.len(), 2);
        assert!(migrated.consumables.is_empty());
        if version >= 4 {
            assert_eq!(migrated.owned_frames, old.owned_frames);
            assert_eq!(migrated.equipped_frame, old.equipped_frame);
        }
        assert_eq!(migrated.exclusive_frame, old.exclusive_frame);
    }
}

#[test]
fn prop_discriminants_remain_compatible_with_released_wallets() {
    for (index, prop) in [
        PropId::ObservationLens,
        PropId::ShengjiCardCounter,
        PropId::ShengjiMissingSuitCard,
        PropId::QiGui523CardCounter,
        PropId::UnoJumpInDevice,
        PropId::MahjongFanCalculator,
        PropId::DoudizhuCardCounter,
        PropId::CardCounter,
        PropId::AutoEggThrower,
    ]
    .into_iter()
    .enumerate()
    {
        assert_eq!(postcard::to_allocvec(&prop).unwrap(), [index as u8]);
    }
}

#[test]
fn reloading_preserves_remaining_consumables_and_their_purchase_cost() {
    let mut wallet = EconomyArchive::new(ProfileId([1; 32]));
    wallet.ledger.coins = 300;
    wallet.buy_consumable(PropId::AutoEggThrower).unwrap();
    wallet.buy_consumable(PropId::AutoEggThrower).unwrap();
    wallet.consume_prop(PropId::AutoEggThrower).unwrap();
    let reloaded = migration::decode(
        FORMAT_VERSION,
        &postcard::to_allocvec(&wallet).unwrap(),
        wallet.ledger.profile_id,
        None,
        0,
    )
    .unwrap();
    assert_eq!(reloaded.ledger.coins, 100);
    assert_eq!(reloaded.consumables[&PropId::AutoEggThrower], 1);
}
