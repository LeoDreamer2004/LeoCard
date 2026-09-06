use super::*;
use crate::{ShengjiThrowPenalty, ShengjiTrump};

fn card(deck: u8, suit: ShengjiSuit, rank: ShengjiRank) -> ShengjiCard {
    ShengjiCard::suited(deck, suit, rank)
}

fn pair(suit: ShengjiSuit, rank: ShengjiRank) -> [ShengjiCard; 2] {
    [card(0, suit, rank), card(1, suit, rank)]
}

fn triple(suit: ShengjiSuit, rank: ShengjiRank) -> [ShengjiCard; 3] {
    [
        card(0, suit, rank),
        card(1, suit, rank),
        card(2, suit, rank),
    ]
}

fn quad(suit: ShengjiSuit, rank: ShengjiRank) -> [ShengjiCard; 4] {
    [
        card(0, suit, rank),
        card(1, suit, rank),
        card(2, suit, rank),
        card(3, suit, rank),
    ]
}

fn trump() -> ShengjiTrump {
    ShengjiTrump::new(ShengjiRank::Ten, Some(ShengjiSuit::Heart)).unwrap()
}

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
fn four_identical_faces_form_bombs_and_adjacent_bombs_form_spaceships() {
    let bombs = [
        quad(ShengjiSuit::Spade, ShengjiRank::Three).as_slice(),
        quad(ShengjiSuit::Spade, ShengjiRank::Four).as_slice(),
    ]
    .concat();
    let play = classify_cards(&bombs, trump()).unwrap();
    assert!(matches!(
        play.components.as_slice(),
        [Component::Spaceship { quad_count: 2, .. }]
    ));
    assert_eq!(play.kitty_multiplier(), 64);

    let single_bomb = classify_cards(&quad(ShengjiSuit::Spade, ShengjiRank::Ace), trump()).unwrap();
    assert!(matches!(
        single_bomb.components.as_slice(),
        [Component::Quad { .. }]
    ));
    assert_eq!(single_bomb.kitty_multiplier(), 16);
}

#[test]
fn equal_strength_off_suit_level_cards_never_merge_into_a_bomb() {
    let no_trump = ShengjiTrump::new(ShengjiRank::Ten, None).unwrap();
    let mixed_faces = [
        pair(ShengjiSuit::Spade, ShengjiRank::Ten).as_slice(),
        pair(ShengjiSuit::Heart, ShengjiRank::Ten).as_slice(),
    ]
    .concat();
    let play = classify_cards(&mixed_faces, no_trump).unwrap();
    assert_eq!(play.components.len(), 2);
    assert!(
        play.components
            .iter()
            .all(|component| matches!(component, Component::Pair { .. }))
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
fn a_led_bomb_requires_a_bomb_before_a_tractor() {
    let lead = classify_cards(&quad(ShengjiSuit::Spade, ShengjiRank::Ace), trump()).unwrap();
    let bomb = quad(ShengjiSuit::Spade, ShengjiRank::Three);
    let tractor_cards = [
        pair(ShengjiSuit::Spade, ShengjiRank::Six).as_slice(),
        pair(ShengjiSuit::Spade, ShengjiRank::Seven).as_slice(),
    ]
    .concat();
    let hand = [bomb.as_slice(), tractor_cards.as_slice()].concat();
    assert_eq!(
        validate_follow(&hand, &tractor_cards, &lead, trump()),
        Err(FollowError::MustFollowStructure)
    );
    assert!(validate_follow(&hand, &bomb, &lead, trump()).is_ok());
}

#[test]
fn spaceship_cross_beats_a_four_pair_tractor_and_is_mandatory() {
    let lead_cards = [
        pair(ShengjiSuit::Spade, ShengjiRank::Three).as_slice(),
        pair(ShengjiSuit::Spade, ShengjiRank::Four).as_slice(),
        pair(ShengjiSuit::Spade, ShengjiRank::Five).as_slice(),
        pair(ShengjiSuit::Spade, ShengjiRank::Six).as_slice(),
    ]
    .concat();
    let lead = classify_cards(&lead_cards, trump()).unwrap();
    let spaceship = [
        quad(ShengjiSuit::Spade, ShengjiRank::Seven).as_slice(),
        quad(ShengjiSuit::Spade, ShengjiRank::Eight).as_slice(),
    ]
    .concat();
    let lower_shape = [
        pair(ShengjiSuit::Spade, ShengjiRank::Jack).as_slice(),
        pair(ShengjiSuit::Spade, ShengjiRank::Queen).as_slice(),
        pair(ShengjiSuit::Spade, ShengjiRank::King).as_slice(),
        pair(ShengjiSuit::Spade, ShengjiRank::Ace).as_slice(),
    ]
    .concat();
    let hand = [spaceship.as_slice(), lower_shape.as_slice()].concat();
    assert_eq!(
        validate_follow(&hand, &lower_shape, &lead, trump()),
        Err(FollowError::MustFollowStructure)
    );
    let spaceship_play = validate_follow(&hand, &spaceship, &lead, trump()).unwrap();
    assert_eq!(
        compare_for_trick(&lead, &lead, &spaceship_play, trump()),
        Ordering::Greater
    );

    let attempted = [
        lead_cards.as_slice(),
        &[card(0, ShengjiSuit::Spade, ShengjiRank::Ace)],
    ]
    .concat();
    let TrickPlay::ThrowFailed(failure) = classify_lead(
        &attempted,
        trump(),
        &ShengjiRuleSet::default(),
        &[&spaceship],
    )
    .unwrap() else {
        panic!("甩出的四连对应被宇宙飞船击破");
    };
    assert!(matches!(
        failure.forced.components.as_slice(),
        [Component::Tractor { pair_count: 4, .. }]
    ));
}

#[test]
fn four_pair_tractor_follow_hierarchy_keeps_the_declared_order() {
    let hierarchy = eight_card_tractor_hierarchy();
    let spaceship = [
        quad(ShengjiSuit::Spade, ShengjiRank::Three).as_slice(),
        quad(ShengjiSuit::Spade, ShengjiRank::Four).as_slice(),
    ]
    .concat();
    let titanic_pair = [
        triple(ShengjiSuit::Spade, ShengjiRank::Three).as_slice(),
        triple(ShengjiSuit::Spade, ShengjiRank::Four).as_slice(),
        pair(ShengjiSuit::Spade, ShengjiRank::Eight).as_slice(),
    ]
    .concat();
    let bomb_pairs = [
        quad(ShengjiSuit::Spade, ShengjiRank::Two).as_slice(),
        pair(ShengjiSuit::Spade, ShengjiRank::Four).as_slice(),
        pair(ShengjiSuit::Spade, ShengjiRank::Six).as_slice(),
    ]
    .concat();
    let triples = [
        triple(ShengjiSuit::Spade, ShengjiRank::Two).as_slice(),
        triple(ShengjiSuit::Spade, ShengjiRank::Four).as_slice(),
        &[
            card(0, ShengjiSuit::Spade, ShengjiRank::Six),
            card(0, ShengjiSuit::Spade, ShengjiRank::Eight),
        ],
    ]
    .concat();
    let four_pairs = [
        ShengjiRank::Two,
        ShengjiRank::Four,
        ShengjiRank::Six,
        ShengjiRank::Eight,
    ]
    .into_iter()
    .flat_map(|rank| pair(ShengjiSuit::Spade, rank))
    .collect::<Vec<_>>();
    let singles = [
        ShengjiRank::Two,
        ShengjiRank::Three,
        ShengjiRank::Four,
        ShengjiRank::Five,
        ShengjiRank::Six,
        ShengjiRank::Seven,
        ShengjiRank::Eight,
        ShengjiRank::Nine,
    ]
    .map(|rank| card(0, ShengjiSuit::Spade, rank));

    assert_eq!(
        best_follow_tier(&spaceship, trump(), &hierarchy, true),
        Some(0)
    );
    assert_eq!(
        best_follow_tier(&titanic_pair, trump(), &hierarchy, true),
        Some(1)
    );
    assert_eq!(
        best_follow_tier(&bomb_pairs, trump(), &hierarchy, true),
        Some(6)
    );
    assert_eq!(
        best_follow_tier(&triples, trump(), &hierarchy, true),
        Some(9)
    );
    assert_eq!(
        best_follow_tier(&four_pairs, trump(), &hierarchy, true),
        Some(13)
    );
    assert_eq!(
        best_follow_tier(&singles, trump(), &hierarchy, true),
        Some(17)
    );
}

#[test]
fn two_bombs_are_a_structured_lead_and_spaceship_has_first_follow_priority() {
    let lead_cards = [
        quad(ShengjiSuit::Spade, ShengjiRank::Two).as_slice(),
        quad(ShengjiSuit::Spade, ShengjiRank::Four).as_slice(),
    ]
    .concat();
    let lead = classify_cards(&lead_cards, trump()).unwrap();
    assert!(!lead.is_throw());
    let spaceship = [
        quad(ShengjiSuit::Spade, ShengjiRank::Six).as_slice(),
        quad(ShengjiSuit::Spade, ShengjiRank::Seven).as_slice(),
    ]
    .concat();
    let other_bombs = [
        quad(ShengjiSuit::Spade, ShengjiRank::Eight).as_slice(),
        quad(ShengjiSuit::Spade, ShengjiRank::Jack).as_slice(),
    ]
    .concat();
    let hand = [spaceship.as_slice(), other_bombs.as_slice()].concat();
    assert_eq!(
        validate_follow(&hand, &other_bombs, &lead, trump()),
        Err(FollowError::MustFollowStructure)
    );
    let spaceship_play = validate_follow(&hand, &spaceship, &lead, trump()).unwrap();
    assert_eq!(
        compare_for_trick(&lead, &lead, &spaceship_play, trump()),
        Ordering::Greater
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
fn two_link_titanic_and_three_pair_tractor_are_optional_but_titanic_wins() {
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
    let ordinary_tractor = [
        pair(ShengjiSuit::Spade, ShengjiRank::Four).as_slice(),
        pair(ShengjiSuit::Spade, ShengjiRank::Five).as_slice(),
        pair(ShengjiSuit::Spade, ShengjiRank::Six).as_slice(),
    ]
    .concat();
    let hand = [titanic.as_slice(), ordinary_tractor.as_slice()].concat();

    assert!(validate_follow(&hand, &ordinary_tractor, &lead, trump()).is_ok());
    let titanic_play = validate_follow(&hand, &titanic, &lead, trump()).unwrap();
    assert!(forced_follow_cards(&hand, &lead, trump()).is_empty());
    let suggestions = follow_suggestions(&hand, &lead, trump(), 8);
    assert!(suggestions.iter().any(|play| matches!(
        play.components.as_slice(),
        [Component::Tractor { pair_count: 3, .. }]
    )));
    assert!(suggestions.iter().any(|play| matches!(
        play.components.as_slice(),
        [Component::Titanic {
            triple_count: 2,
            ..
        }]
    )));
    assert_eq!(
        compare_for_trick(&lead, &lead, &titanic_play, trump()),
        Ordering::Greater
    );

    let attempted = [
        lead_cards.as_slice(),
        &[card(0, ShengjiSuit::Spade, ShengjiRank::Ace)],
    ]
    .concat();
    let TrickPlay::ThrowFailed(failure) =
        classify_lead(&attempted, trump(), &ShengjiRuleSet::default(), &[&titanic]).unwrap()
    else {
        panic!("甩出的三连对拖拉机应被泰坦尼克击破");
    };
    assert!(matches!(
        failure.forced.components.as_slice(),
        [Component::Tractor { pair_count: 3, .. }]
    ));
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
fn failed_throw_selects_the_shortest_beatable_then_weakest_component() {
    let attempted = [
        card(0, ShengjiSuit::Spade, ShengjiRank::Nine),
        pair(ShengjiSuit::Spade, ShengjiRank::Eight)[0],
        pair(ShengjiSuit::Spade, ShengjiRank::Eight)[1],
        pair(ShengjiSuit::Spade, ShengjiRank::Four)[0],
        pair(ShengjiSuit::Spade, ShengjiRank::Four)[1],
    ];
    let opponent = [
        card(0, ShengjiSuit::Spade, ShengjiRank::Jack),
        pair(ShengjiSuit::Spade, ShengjiRank::Five)[0],
    ];
    let rules = ShengjiRuleSet {
        throw_penalty: ShengjiThrowPenalty::TenPerCard,
        ..ShengjiRuleSet::default()
    };
    let TrickPlay::ThrowFailed(failure) =
        classify_lead(&attempted, trump(), &rules, &[&opponent]).unwrap()
    else {
        panic!("throw should fail");
    };
    assert_eq!(failure.forced.cards, vec![attempted[0]]);
    assert_eq!(failure.penalty_points, 50);
}

#[test]
fn failed_throw_uses_a_pair_when_its_single_is_unbeatable() {
    let attempted = [
        card(0, ShengjiSuit::Spade, ShengjiRank::Ace),
        pair(ShengjiSuit::Spade, ShengjiRank::Eight)[0],
        pair(ShengjiSuit::Spade, ShengjiRank::Eight)[1],
        pair(ShengjiSuit::Spade, ShengjiRank::Four)[0],
        pair(ShengjiSuit::Spade, ShengjiRank::Four)[1],
    ];
    let opponent = pair(ShengjiSuit::Spade, ShengjiRank::Five);
    let TrickPlay::ThrowFailed(failure) = classify_lead(
        &attempted,
        trump(),
        &ShengjiRuleSet::default(),
        &[&opponent],
    )
    .unwrap() else {
        panic!("the low pair should make the throw fail");
    };
    assert_eq!(
        failure.forced.cards,
        pair(ShengjiSuit::Spade, ShengjiRank::Four)
    );
}

#[test]
fn pair_and_tractor_follow_obligations_are_enforced() {
    let lead_pair = classify_cards(&pair(ShengjiSuit::Spade, ShengjiRank::Five), trump()).unwrap();
    let hand = [
        pair(ShengjiSuit::Spade, ShengjiRank::Four).as_slice(),
        &[card(0, ShengjiSuit::Spade, ShengjiRank::Ace)],
    ]
    .concat();
    assert_eq!(
        validate_follow(
            &hand,
            &[
                card(0, ShengjiSuit::Spade, ShengjiRank::Ace),
                pair(ShengjiSuit::Spade, ShengjiRank::Four)[0]
            ],
            &lead_pair,
            trump()
        ),
        Err(FollowError::MustFollowStructure)
    );

    let lead_tractor = classify_cards(
        &[
            pair(ShengjiSuit::Spade, ShengjiRank::King),
            pair(ShengjiSuit::Spade, ShengjiRank::Queen),
        ]
        .concat(),
        trump(),
    )
    .unwrap();
    let follow = [
        pair(ShengjiSuit::Spade, ShengjiRank::Eight),
        pair(ShengjiSuit::Spade, ShengjiRank::Seven),
    ]
    .concat();
    let hand = [
        follow.clone(),
        pair(ShengjiSuit::Spade, ShengjiRank::Four).to_vec(),
    ]
    .concat();
    assert!(validate_follow(&hand, &follow, &lead_tractor, trump()).is_ok());
}

#[test]
fn forced_follow_cards_selects_the_only_pair_and_all_short_suit_cards() {
    let lead_tractor = classify_cards(
        &[
            pair(ShengjiSuit::Spade, ShengjiRank::Jack),
            pair(ShengjiSuit::Spade, ShengjiRank::Queen),
        ]
        .concat(),
        trump(),
    )
    .unwrap();
    let threes = pair(ShengjiSuit::Spade, ShengjiRank::Three);
    let hand = [
        threes.as_slice(),
        &[
            card(0, ShengjiSuit::Spade, ShengjiRank::Six),
            card(0, ShengjiSuit::Spade, ShengjiRank::Seven),
            card(0, ShengjiSuit::Spade, ShengjiRank::Nine),
        ],
    ]
    .concat();
    assert_eq!(forced_follow_cards(&hand, &lead_tractor, trump()), threes);
    let suggestions = follow_suggestions(&hand, &lead_tractor, trump(), 16);
    assert_eq!(suggestions.len(), 3);
    assert!(suggestions.iter().all(|play| {
        threes
            .iter()
            .all(|card| play.cards.iter().any(|candidate| candidate == card))
    }));
    assert!(
        suggestions
            .windows(2)
            .all(|window| window[0].cards != window[1].cards)
    );

    let lead_throw = classify_cards(
        &[
            card(0, ShengjiSuit::Spade, ShengjiRank::Ace),
            pair(ShengjiSuit::Spade, ShengjiRank::Jack)[0],
            pair(ShengjiSuit::Spade, ShengjiRank::Jack)[1],
            pair(ShengjiSuit::Spade, ShengjiRank::Queen)[0],
            pair(ShengjiSuit::Spade, ShengjiRank::Queen)[1],
        ],
        trump(),
    )
    .unwrap();
    assert_eq!(forced_follow_cards(&hand, &lead_throw, trump()), hand);
}

#[test]
fn forced_follow_cards_does_not_choose_between_two_valid_tractors() {
    let lead = classify_cards(
        &[
            pair(ShengjiSuit::Spade, ShengjiRank::Jack),
            pair(ShengjiSuit::Spade, ShengjiRank::Queen),
        ]
        .concat(),
        trump(),
    )
    .unwrap();
    let hand = [
        pair(ShengjiSuit::Spade, ShengjiRank::Three).as_slice(),
        pair(ShengjiSuit::Spade, ShengjiRank::Four).as_slice(),
        pair(ShengjiSuit::Spade, ShengjiRank::Seven).as_slice(),
        pair(ShengjiSuit::Spade, ShengjiRank::Eight).as_slice(),
    ]
    .concat();
    assert!(forced_follow_cards(&hand, &lead, trump()).is_empty());
    assert_eq!(follow_suggestions(&hand, &lead, trump(), 16).len(), 2);
}

#[test]
fn a_void_player_may_mix_off_suits_and_the_discard_cannot_win() {
    let lead = classify_cards(
        &[
            card(0, ShengjiSuit::Spade, ShengjiRank::Five),
            card(0, ShengjiSuit::Spade, ShengjiRank::Seven),
        ],
        trump(),
    )
    .unwrap();
    let hand = [
        card(0, ShengjiSuit::Diamond, ShengjiRank::Three),
        card(0, ShengjiSuit::Club, ShengjiRank::Four),
        card(0, ShengjiSuit::Heart, ShengjiRank::Ace),
    ];
    let discard = validate_follow(&hand, &hand[..2], &lead, trump()).unwrap();

    assert_eq!(discard.category, Category::Mixed);
    assert_eq!(
        compare_for_trick(&lead, &lead, &discard, trump()),
        Ordering::Less
    );
}

#[test]
fn a_short_led_suit_may_be_completed_with_another_suit() {
    let lead = classify_cards(
        &[
            card(0, ShengjiSuit::Spade, ShengjiRank::Five),
            card(0, ShengjiSuit::Spade, ShengjiRank::Seven),
        ],
        trump(),
    )
    .unwrap();
    let spade = card(0, ShengjiSuit::Spade, ShengjiRank::Three);
    let club = card(0, ShengjiSuit::Club, ShengjiRank::Four);
    let hand = [spade, club, card(0, ShengjiSuit::Heart, ShengjiRank::Ace)];
    let discard = validate_follow(&hand, &[spade, club], &lead, trump()).unwrap();

    assert_eq!(discard.category, Category::Mixed);
}

#[test]
fn kill_and_overkill_compare_the_structure_required_by_the_lead() {
    let all_singles = classify_cards(
        &[
            card(0, ShengjiSuit::Spade, ShengjiRank::Five),
            card(0, ShengjiSuit::Spade, ShengjiRank::Six),
            card(0, ShengjiSuit::Spade, ShengjiRank::Seven),
        ],
        trump(),
    )
    .unwrap();
    let first_kill = classify_cards(
        &[
            card(0, ShengjiSuit::Heart, ShengjiRank::Nine),
            card(0, ShengjiSuit::Heart, ShengjiRank::Seven),
            card(1, ShengjiSuit::Heart, ShengjiRank::Seven),
        ],
        trump(),
    )
    .unwrap();
    let attempted_cover = classify_cards(
        &[
            card(0, ShengjiSuit::Heart, ShengjiRank::Eight),
            card(1, ShengjiSuit::Heart, ShengjiRank::Eight),
            card(0, ShengjiSuit::Heart, ShengjiRank::Four),
        ],
        trump(),
    )
    .unwrap();
    assert_eq!(
        compare_for_trick(&all_singles, &first_kill, &attempted_cover, trump()),
        Ordering::Less
    );

    let pair_lead = classify_cards(
        &[
            card(0, ShengjiSuit::Spade, ShengjiRank::Five),
            pair(ShengjiSuit::Spade, ShengjiRank::Four)[0],
            pair(ShengjiSuit::Spade, ShengjiRank::Four)[1],
        ],
        trump(),
    )
    .unwrap();
    assert_eq!(
        compare_for_trick(&pair_lead, &first_kill, &attempted_cover, trump()),
        Ordering::Greater
    );
}

#[test]
fn trump_padding_without_the_led_structure_cannot_kill() {
    let lead_cards = [
        pair(ShengjiSuit::Spade, ShengjiRank::Six).as_slice(),
        pair(ShengjiSuit::Spade, ShengjiRank::Seven).as_slice(),
    ]
    .concat();
    let lead = classify_cards(&lead_cards, trump()).unwrap();
    let padding = classify_cards(
        &[
            card(0, ShengjiSuit::Heart, ShengjiRank::Three),
            card(0, ShengjiSuit::Heart, ShengjiRank::Five),
            card(0, ShengjiSuit::Heart, ShengjiRank::Seven),
            card(0, ShengjiSuit::Heart, ShengjiRank::Nine),
        ],
        trump(),
    )
    .unwrap();
    assert_eq!(
        compare_for_trick(&lead, &lead, &padding, trump()),
        Ordering::Less
    );
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
