use super::*;

#[test]
fn three_deck_triples_form_titanics_across_skipped_and_special_levels() {
    let level_five = ShengjiTrump::new(ShengjiRank::Five, Some(ShengjiSuit::Heart)).unwrap();
    for cards in [
        [
            triple(ShengjiSuit::Spade, ShengjiRank::Three).as_slice(),
            triple(ShengjiSuit::Spade, ShengjiRank::Four).as_slice(),
        ]
        .concat(),
        [
            triple(ShengjiSuit::Spade, ShengjiRank::Four).as_slice(),
            triple(ShengjiSuit::Spade, ShengjiRank::Six).as_slice(),
        ]
        .concat(),
        [
            triple(ShengjiSuit::Heart, ShengjiRank::Ace).as_slice(),
            triple(ShengjiSuit::Spade, ShengjiRank::Five).as_slice(),
        ]
        .concat(),
        [
            triple(ShengjiSuit::Spade, ShengjiRank::Five).as_slice(),
            triple(ShengjiSuit::Heart, ShengjiRank::Five).as_slice(),
        ]
        .concat(),
        [
            triple(ShengjiSuit::Heart, ShengjiRank::Five).as_slice(),
            &[
                ShengjiCard::small_joker(0),
                ShengjiCard::small_joker(1),
                ShengjiCard::small_joker(2),
            ],
        ]
        .concat(),
    ] {
        let play = classify_cards(&cards, level_five).unwrap();
        assert!(matches!(
            play.components.as_slice(),
            [Component::Titanic {
                triple_count: 2,
                ..
            }]
        ));
        assert_eq!(play.kitty_multiplier(), 16);
    }

    let split_category = [
        triple(ShengjiSuit::Spade, ShengjiRank::Five).as_slice(),
        triple(ShengjiSuit::Spade, ShengjiRank::Six).as_slice(),
    ]
    .concat();
    assert_eq!(
        classify_cards(&split_category, level_five),
        Err(PlayError::MixedCategory)
    );

    let no_trump = ShengjiTrump::new(ShengjiRank::Five, None).unwrap();
    let same_strength = [
        triple(ShengjiSuit::Diamond, ShengjiRank::Five).as_slice(),
        triple(ShengjiSuit::Club, ShengjiRank::Five).as_slice(),
    ]
    .concat();
    let play = classify_cards(&same_strength, no_trump).unwrap();
    assert!(
        play.components
            .iter()
            .all(|component| matches!(component, Component::Triple { .. }))
    );
}

#[test]
fn bomb_and_two_pair_tractor_share_follow_tier_but_bomb_wins() {
    let lead_cards = [
        pair(ShengjiSuit::Spade, ShengjiRank::Six).as_slice(),
        pair(ShengjiSuit::Spade, ShengjiRank::Seven).as_slice(),
    ]
    .concat();
    let lead = classify_cards(&lead_cards, trump()).unwrap();
    let bomb = quad(ShengjiSuit::Spade, ShengjiRank::Three);
    let tractor_cards = [
        pair(ShengjiSuit::Spade, ShengjiRank::Eight).as_slice(),
        pair(ShengjiSuit::Spade, ShengjiRank::Nine).as_slice(),
    ]
    .concat();
    let hand = [bomb.as_slice(), tractor_cards.as_slice()].concat();
    let bomb_play = validate_follow(&hand, &bomb, &lead, trump()).unwrap();
    let tractor_play = validate_follow(&hand, &tractor_cards, &lead, trump()).unwrap();
    assert_eq!(
        compare_for_trick(&lead, &tractor_play, &bomb_play, trump()),
        Ordering::Greater
    );
    assert_eq!(
        compare_for_trick(&lead, &bomb_play, &tractor_play, trump()),
        Ordering::Less
    );
}

#[test]
fn bomb_plus_pair_is_below_tractor_and_titanic_for_six_card_following() {
    let lead_cards = [
        pair(ShengjiSuit::Spade, ShengjiRank::Six).as_slice(),
        pair(ShengjiSuit::Spade, ShengjiRank::Seven).as_slice(),
        pair(ShengjiSuit::Spade, ShengjiRank::Eight).as_slice(),
    ]
    .concat();
    let lead = classify_cards(&lead_cards, trump()).unwrap();
    let titanic = [
        triple(ShengjiSuit::Spade, ShengjiRank::Two).as_slice(),
        triple(ShengjiSuit::Spade, ShengjiRank::Three).as_slice(),
    ]
    .concat();
    let bomb_plus_pair = [
        quad(ShengjiSuit::Spade, ShengjiRank::King).as_slice(),
        pair(ShengjiSuit::Spade, ShengjiRank::Ace).as_slice(),
    ]
    .concat();
    let hand = [titanic.as_slice(), bomb_plus_pair.as_slice()].concat();
    assert_eq!(
        validate_follow(&hand, &bomb_plus_pair, &lead, trump()),
        Err(FollowError::MustFollowStructure)
    );
    assert!(validate_follow(&hand, &titanic, &lead, trump()).is_ok());
}

#[test]
fn triple_and_titanic_follow_priorities_are_enforced() {
    let lead_triple =
        classify_cards(&triple(ShengjiSuit::Spade, ShengjiRank::King), trump()).unwrap();
    let sixes = triple(ShengjiSuit::Spade, ShengjiRank::Six);
    let twos = pair(ShengjiSuit::Spade, ShengjiRank::Two);
    let triple_hand = [sixes.as_slice(), twos.as_slice()].concat();
    let pair_follow = [twos.as_slice(), &sixes[..1]].concat();
    assert_eq!(
        validate_follow(&triple_hand, &pair_follow, &lead_triple, trump()),
        Err(FollowError::MustFollowStructure)
    );
    assert!(validate_follow(&triple_hand, &sixes, &lead_triple, trump()).is_ok());

    let lead_titanic_cards = [
        triple(ShengjiSuit::Spade, ShengjiRank::Three).as_slice(),
        triple(ShengjiSuit::Spade, ShengjiRank::Four).as_slice(),
    ]
    .concat();
    let lead_titanic = classify_cards(&lead_titanic_cards, trump()).unwrap();
    let six_pair = pair(ShengjiSuit::Spade, ShengjiRank::Six);
    let seven_pair = pair(ShengjiSuit::Spade, ShengjiRank::Seven);
    let two_triple = triple(ShengjiSuit::Spade, ShengjiRank::Two);
    let eight_triple = triple(ShengjiSuit::Spade, ShengjiRank::Eight);
    let hand = [
        six_pair.as_slice(),
        seven_pair.as_slice(),
        two_triple.as_slice(),
        eight_triple.as_slice(),
    ]
    .concat();
    let two_triples = [two_triple.as_slice(), eight_triple.as_slice()].concat();
    assert_eq!(
        validate_follow(&hand, &two_triples, &lead_titanic, trump()),
        Err(FollowError::MustFollowStructure)
    );
    let tractor_and_singles = [
        six_pair.as_slice(),
        seven_pair.as_slice(),
        &two_triple[..1],
        &eight_triple[..1],
    ]
    .concat();
    assert!(validate_follow(&hand, &tractor_and_singles, &lead_titanic, trump()).is_ok());
}

#[test]
fn level_is_skipped_and_special_trump_pairs_form_tractors() {
    let jj99 = [
        pair(ShengjiSuit::Spade, ShengjiRank::Jack),
        pair(ShengjiSuit::Spade, ShengjiRank::Nine),
    ]
    .concat();
    let play = classify_cards(&jj99, trump()).unwrap();
    assert!(matches!(
        play.components.as_slice(),
        [Component::Tractor { pair_count: 2, .. }]
    ));

    let special = [
        pair(ShengjiSuit::Heart, ShengjiRank::Ace).as_slice(),
        pair(ShengjiSuit::Spade, ShengjiRank::Ten).as_slice(),
        pair(ShengjiSuit::Heart, ShengjiRank::Ten).as_slice(),
        &[ShengjiCard::small_joker(0), ShengjiCard::small_joker(1)],
        &[ShengjiCard::big_joker(0), ShengjiCard::big_joker(1)],
    ]
    .concat();
    let play = classify_cards(&special, trump()).unwrap();
    assert!(matches!(
        play.components.as_slice(),
        [Component::Tractor { pair_count: 5, .. }]
    ));
}

#[test]
fn constant_twos_extend_the_special_trump_tractor_chain() {
    let trump = trump().with_constant_trump(true);
    let special = [
        pair(ShengjiSuit::Heart, ShengjiRank::Ace).as_slice(),
        pair(ShengjiSuit::Spade, ShengjiRank::Two).as_slice(),
        pair(ShengjiSuit::Heart, ShengjiRank::Two).as_slice(),
        pair(ShengjiSuit::Spade, ShengjiRank::Ten).as_slice(),
        pair(ShengjiSuit::Heart, ShengjiRank::Ten).as_slice(),
        &[ShengjiCard::small_joker(0), ShengjiCard::small_joker(1)],
        &[ShengjiCard::big_joker(0), ShengjiCard::big_joker(1)],
    ]
    .concat();
    assert!(matches!(
        classify_cards(&special, trump)
            .unwrap()
            .components
            .as_slice(),
        [Component::Tractor { pair_count: 7, .. }]
    ));
}

#[test]
fn no_trump_small_jokers_connect_level_pairs_and_big_jokers() {
    let no_trump = ShengjiTrump::new(ShengjiRank::Ten, None).unwrap();
    let cards = [
        pair(ShengjiSuit::Diamond, ShengjiRank::Ten).as_slice(),
        &[ShengjiCard::small_joker(0), ShengjiCard::small_joker(1)],
        &[ShengjiCard::big_joker(0), ShengjiCard::big_joker(1)],
    ]
    .concat();
    assert!(matches!(
        classify_cards(&cards, no_trump)
            .unwrap()
            .components
            .as_slice(),
        [Component::Tractor { pair_count: 3, .. }]
    ));
}

#[test]
fn kitty_multiplier_uses_the_strongest_throw_component() {
    let cards = [
        pair(ShengjiSuit::Spade, ShengjiRank::Ace).as_slice(),
        pair(ShengjiSuit::Spade, ShengjiRank::King).as_slice(),
        &[card(0, ShengjiSuit::Spade, ShengjiRank::Queen)],
    ]
    .concat();
    let play = classify_cards(&cards, trump()).unwrap();
    assert_eq!(play.kitty_multiplier(), 8);

    for (pair_count, multiplier) in [(2, 8), (3, 16), (4, 32)] {
        let ranks = [
            ShengjiRank::Ace,
            ShengjiRank::King,
            ShengjiRank::Queen,
            ShengjiRank::Jack,
        ];
        let tractor = ranks[..pair_count]
            .iter()
            .flat_map(|rank| pair(ShengjiSuit::Spade, *rank))
            .collect::<Vec<_>>();
        assert_eq!(
            classify_cards(&tractor, trump())
                .unwrap()
                .kitty_multiplier(),
            multiplier
        );
    }
}
