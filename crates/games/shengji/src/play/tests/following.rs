use super::*;

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
