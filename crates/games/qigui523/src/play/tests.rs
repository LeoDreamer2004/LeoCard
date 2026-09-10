use super::*;
use crate::QiGuiSuit::{Club, Diamond, Heart, Spade};
use crate::{QiGuiCard, QiGuiRank, QiGuiRuleSet, SameCardPolicy, SuitComparison};

fn card(rank: QiGuiRank) -> QiGuiCard {
    QiGuiCard::suited(0, Diamond, rank)
}

#[test]
fn unusual_rank_order_drives_straights() {
    let rules = QiGuiRuleSet::default();
    let four_six_eight_nine = [
        card(QiGuiRank::Four),
        card(QiGuiRank::Six),
        card(QiGuiRank::Eight),
        card(QiGuiRank::Nine),
    ];
    let queen_to_two = [
        card(QiGuiRank::Queen),
        card(QiGuiRank::King),
        card(QiGuiRank::Ace),
        card(QiGuiRank::Three),
        card(QiGuiRank::Two),
    ];
    let ordinary_456 = [
        card(QiGuiRank::Four),
        card(QiGuiRank::Five),
        card(QiGuiRank::Six),
    ];

    assert!(matches!(
        classify(&four_six_eight_nine, &rules).unwrap().kind(),
        QiGuiPlayKind::Straight { card_count: 4 }
    ));
    assert!(matches!(
        classify(&queen_to_two, &rules).unwrap().kind(),
        QiGuiPlayKind::Straight { card_count: 5 }
    ));
    assert_eq!(
        classify(&ordinary_456, &rules),
        Err(PlayError::InvalidPattern)
    );
}

#[test]
fn recognizes_consecutive_pairs() {
    let rules = QiGuiRuleSet {
        deck_count: 2,
        ..QiGuiRuleSet::default()
    };
    let cards = [
        QiGuiCard::suited(0, Diamond, QiGuiRank::Queen),
        QiGuiCard::suited(0, Club, QiGuiRank::Queen),
        QiGuiCard::suited(0, Diamond, QiGuiRank::King),
        QiGuiCard::suited(0, Club, QiGuiRank::King),
    ];
    assert!(matches!(
        classify(&cards, &rules).unwrap().kind(),
        QiGuiPlayKind::ConsecutivePairs { pair_count: 2 }
    ));
}

#[test]
fn exact_duplicate_strength_obeys_room_setting() {
    let strict = QiGuiRuleSet {
        deck_count: 2,
        ..QiGuiRuleSet::default()
    };
    let following = QiGuiRuleSet {
        same_card_policy: SameCardPolicy::CanFollow,
        ..strict
    };
    let first = classify(&[QiGuiCard::suited(0, Spade, QiGuiRank::Ace)], &strict).unwrap();
    let duplicate = classify(&[QiGuiCard::suited(1, Spade, QiGuiRank::Ace)], &strict).unwrap();

    assert_eq!(
        compare_plays(&duplicate, &first, &strict),
        PlayComparison::Equivalent
    );
    assert!(!can_beat(&duplicate, &first, &strict));
    assert!(can_beat(&duplicate, &first, &following));
}

#[test]
fn advanced_play_types_obey_the_three_fixed_suppression_relations() {
    let disabled = QiGuiRuleSet::default();
    let enabled = QiGuiRuleSet {
        advanced_play_types: true,
        ..disabled
    };
    let straight_three = classify(
        &[
            card(QiGuiRank::Eight),
            card(QiGuiRank::Nine),
            card(QiGuiRank::Ten),
        ],
        &enabled,
    )
    .unwrap();
    let triple = classify(
        &[
            QiGuiCard::suited(0, Diamond, QiGuiRank::Four),
            QiGuiCard::suited(0, Club, QiGuiRank::Four),
            QiGuiCard::suited(0, Heart, QiGuiRank::Four),
        ],
        &enabled,
    )
    .unwrap();
    let straight_four = classify(
        &[
            card(QiGuiRank::Eight),
            card(QiGuiRank::Nine),
            card(QiGuiRank::Ten),
            card(QiGuiRank::Jack),
        ],
        &enabled,
    )
    .unwrap();
    let two_pairs = classify(
        &[
            QiGuiCard::suited(0, Diamond, QiGuiRank::Four),
            QiGuiCard::suited(0, Club, QiGuiRank::Four),
            QiGuiCard::suited(0, Diamond, QiGuiRank::Six),
            QiGuiCard::suited(0, Club, QiGuiRank::Six),
        ],
        &enabled,
    )
    .unwrap();
    let three_pairs = classify(
        &[
            QiGuiCard::suited(0, Diamond, QiGuiRank::Eight),
            QiGuiCard::suited(0, Club, QiGuiRank::Eight),
            QiGuiCard::suited(0, Diamond, QiGuiRank::Nine),
            QiGuiCard::suited(0, Club, QiGuiRank::Nine),
            QiGuiCard::suited(0, Diamond, QiGuiRank::Ten),
            QiGuiCard::suited(0, Club, QiGuiRank::Ten),
        ],
        &enabled,
    )
    .unwrap();
    let two_plane = classify(
        &[
            QiGuiCard::suited(0, Diamond, QiGuiRank::Four),
            QiGuiCard::suited(0, Club, QiGuiRank::Four),
            QiGuiCard::suited(0, Heart, QiGuiRank::Four),
            QiGuiCard::suited(0, Diamond, QiGuiRank::Six),
            QiGuiCard::suited(0, Club, QiGuiRank::Six),
            QiGuiCard::suited(0, Heart, QiGuiRank::Six),
        ],
        &enabled,
    )
    .unwrap();

    assert_eq!(
        compare_plays(&triple, &straight_three, &disabled),
        PlayComparison::Incompatible
    );
    for (stronger, weaker) in [
        (&triple, &straight_three),
        (&two_pairs, &straight_four),
        (&two_plane, &three_pairs),
    ] {
        assert_eq!(
            compare_plays(stronger, weaker, &enabled),
            PlayComparison::Greater
        );
        assert_eq!(
            compare_plays(weaker, stronger, &enabled),
            PlayComparison::Lower
        );
        assert!(can_beat(stronger, weaker, &enabled));
        assert!(!can_beat(weaker, stronger, &enabled));
    }
}

#[test]
fn advanced_triple_carries_compare_only_the_triple_component() {
    let disabled = QiGuiRuleSet::default();
    let enabled = QiGuiRuleSet {
        advanced_play_types: true,
        ..disabled
    };
    let triple_four = [
        QiGuiCard::suited(0, Diamond, QiGuiRank::Four),
        QiGuiCard::suited(0, Club, QiGuiRank::Four),
        QiGuiCard::suited(0, Heart, QiGuiRank::Four),
    ];
    let low_kicker_cards = [
        triple_four[0],
        triple_four[1],
        triple_four[2],
        card(QiGuiRank::Six),
    ];
    let high_kicker_cards = [
        triple_four[0],
        triple_four[1],
        triple_four[2],
        card(QiGuiRank::Seven),
    ];
    assert_eq!(
        classify(&low_kicker_cards, &disabled),
        Err(PlayError::InvalidPattern)
    );
    let low_kicker = classify(&low_kicker_cards, &enabled).unwrap();
    let high_kicker = classify(&high_kicker_cards, &enabled).unwrap();
    assert_eq!(low_kicker.kind(), &QiGuiPlayKind::TripleWithSingle);
    assert_eq!(
        compare_plays(&high_kicker, &low_kicker, &enabled),
        PlayComparison::Equivalent
    );
    assert!(!can_beat(&high_kicker, &low_kicker, &enabled));
    let following = QiGuiRuleSet {
        same_card_policy: SameCardPolicy::CanFollow,
        ..enabled
    };
    assert!(can_beat(&high_kicker, &low_kicker, &following));

    let lower_triple_with_pair = classify(
        &[
            triple_four[0],
            triple_four[1],
            triple_four[2],
            QiGuiCard::suited(0, Diamond, QiGuiRank::Seven),
            QiGuiCard::suited(0, Club, QiGuiRank::Seven),
        ],
        &enabled,
    )
    .unwrap();
    let higher_triple_with_pair = classify(
        &[
            QiGuiCard::suited(0, Diamond, QiGuiRank::Six),
            QiGuiCard::suited(0, Club, QiGuiRank::Six),
            QiGuiCard::suited(0, Heart, QiGuiRank::Six),
            QiGuiCard::suited(0, Diamond, QiGuiRank::Four),
            QiGuiCard::suited(0, Club, QiGuiRank::Four),
        ],
        &enabled,
    )
    .unwrap();
    assert_eq!(
        lower_triple_with_pair.kind(),
        &QiGuiPlayKind::TripleWithPair
    );
    assert_eq!(
        compare_plays(&higher_triple_with_pair, &lower_triple_with_pair, &enabled),
        PlayComparison::Greater
    );

    let straight = classify(
        &[
            card(QiGuiRank::Eight),
            card(QiGuiRank::Nine),
            card(QiGuiRank::Ten),
        ],
        &enabled,
    )
    .unwrap();
    assert_eq!(
        compare_plays(&high_kicker, &straight, &enabled),
        PlayComparison::Incompatible
    );
    assert!(!can_beat(&high_kicker, &straight, &enabled));
}

#[test]
fn suit_breaks_a_same_rank_tie() {
    let rules = QiGuiRuleSet::default();
    let diamond = classify(&[QiGuiCard::suited(0, Diamond, QiGuiRank::Four)], &rules).unwrap();
    let spade = classify(&[QiGuiCard::suited(0, Spade, QiGuiRank::Four)], &rules).unwrap();

    assert_eq!(
        compare_plays(&spade, &diamond, &rules),
        PlayComparison::Greater
    );
}

#[test]
fn all_three_suit_comparison_modes_are_distinct() {
    let highest = QiGuiRuleSet {
        deck_count: 2,
        suit_comparison: SuitComparison::HighestCard,
        ..QiGuiRuleSet::default()
    };
    let left = classify(
        &[
            QiGuiCard::suited(0, Spade, QiGuiRank::Four),
            QiGuiCard::suited(0, Diamond, QiGuiRank::Four),
        ],
        &highest,
    )
    .unwrap();
    let right = classify(
        &[
            QiGuiCard::suited(1, Spade, QiGuiRank::Four),
            QiGuiCard::suited(0, Club, QiGuiRank::Four),
        ],
        &highest,
    )
    .unwrap();

    assert_eq!(
        compare_plays(&right, &left, &highest),
        PlayComparison::Equivalent
    );

    let lexicographic = QiGuiRuleSet {
        suit_comparison: SuitComparison::Lexicographic,
        ..highest
    };
    assert_eq!(
        compare_plays(&right, &left, &lexicographic),
        PlayComparison::Greater
    );

    let sum_points = QiGuiRuleSet {
        suit_comparison: SuitComparison::SumPoints,
        ..highest
    };
    assert_eq!(
        compare_plays(&right, &left, &sum_points),
        PlayComparison::Greater
    );

    let spade_diamond = classify(
        &[
            QiGuiCard::suited(0, Spade, QiGuiRank::Six),
            QiGuiCard::suited(0, Diamond, QiGuiRank::Six),
        ],
        &highest,
    )
    .unwrap();
    let heart_club = classify(
        &[
            QiGuiCard::suited(0, Heart, QiGuiRank::Six),
            QiGuiCard::suited(0, Club, QiGuiRank::Six),
        ],
        &highest,
    )
    .unwrap();
    assert_eq!(
        compare_plays(&spade_diamond, &heart_club, &lexicographic),
        PlayComparison::Greater
    );
    assert_eq!(
        compare_plays(&spade_diamond, &heart_club, &sum_points),
        PlayComparison::Equivalent
    );
}


#[test]
fn recognizes_airplanes_and_only_allows_matching_airplanes_to_follow() {
    let rules = QiGuiRuleSet::default();
    let airplane = classify(
        &[
            QiGuiCard::suited(0, Diamond, QiGuiRank::Six),
            QiGuiCard::suited(0, Club, QiGuiRank::Six),
            QiGuiCard::suited(0, Heart, QiGuiRank::Six),
            QiGuiCard::suited(0, Diamond, QiGuiRank::Eight),
            QiGuiCard::suited(0, Club, QiGuiRank::Eight),
            QiGuiCard::suited(0, Heart, QiGuiRank::Eight),
        ],
        &rules,
    )
    .unwrap();
    let higher_airplane = classify(
        &[
            QiGuiCard::suited(0, Diamond, QiGuiRank::Eight),
            QiGuiCard::suited(0, Club, QiGuiRank::Eight),
            QiGuiCard::suited(0, Heart, QiGuiRank::Eight),
            QiGuiCard::suited(0, Diamond, QiGuiRank::Nine),
            QiGuiCard::suited(0, Club, QiGuiRank::Nine),
            QiGuiCard::suited(0, Heart, QiGuiRank::Nine),
        ],
        &rules,
    )
    .unwrap();
    let straight = classify(
        &[
            card(QiGuiRank::Four),
            card(QiGuiRank::Six),
            card(QiGuiRank::Eight),
            card(QiGuiRank::Nine),
            card(QiGuiRank::Ten),
            card(QiGuiRank::Jack),
        ],
        &rules,
    )
    .unwrap();

    assert_eq!(
        airplane.kind(),
        &QiGuiPlayKind::Airplane { triple_count: 2 }
    );
    assert_eq!(
        compare_plays(&higher_airplane, &airplane, &rules),
        PlayComparison::Greater
    );
    assert_eq!(
        compare_plays(&airplane, &straight, &rules),
        PlayComparison::Incompatible
    );
}

#[test]
fn heaven_bomb_beats_every_other_shape_and_obeys_same_play_tie_rules() {
    let strict = QiGuiRuleSet {
        deck_count: 2,
        ..QiGuiRuleSet::default()
    };
    let heaven_cards = |deck, joker_suit| {
        [
            QiGuiCard::suited(deck, Diamond, QiGuiRank::Three),
            QiGuiCard::suited(deck, Diamond, QiGuiRank::Two),
            QiGuiCard::suited(deck, Diamond, QiGuiRank::Five),
            QiGuiCard::suited(deck, joker_suit, QiGuiRank::Joker),
            QiGuiCard::suited(deck, Diamond, QiGuiRank::Seven),
        ]
    };
    let heaven = classify(&heaven_cards(0, Club), &strict).unwrap();
    let physical_copy = classify(&heaven_cards(1, Club), &strict).unwrap();
    let ordinary_bomb = classify(
        &[
            QiGuiCard::suited(0, Diamond, QiGuiRank::Seven),
            QiGuiCard::suited(0, Club, QiGuiRank::Seven),
            QiGuiCard::suited(0, Heart, QiGuiRank::Seven),
            QiGuiCard::suited(0, Spade, QiGuiRank::Seven),
        ],
        &strict,
    )
    .unwrap();

    assert_eq!(heaven.kind(), &QiGuiPlayKind::HeavenBomb);
    assert_eq!(
        compare_plays(&heaven, &ordinary_bomb, &strict),
        PlayComparison::Greater
    );
    assert_eq!(
        compare_plays(&ordinary_bomb, &heaven, &strict),
        PlayComparison::Lower
    );
    assert_eq!(
        compare_plays(&physical_copy, &heaven, &strict),
        PlayComparison::Equivalent
    );
    assert!(!can_beat(&physical_copy, &heaven, &strict));
    let following = QiGuiRuleSet {
        same_card_policy: SameCardPolicy::CanFollow,
        ..strict
    };
    assert!(can_beat(&physical_copy, &heaven, &following));

    let stronger_suit = classify(
        &[
            QiGuiCard::suited(0, Diamond, QiGuiRank::Three),
            QiGuiCard::suited(0, Diamond, QiGuiRank::Two),
            QiGuiCard::suited(0, Diamond, QiGuiRank::Five),
            QiGuiCard::suited(0, Spade, QiGuiRank::Joker),
            QiGuiCard::suited(0, Spade, QiGuiRank::Seven),
        ],
        &strict,
    )
    .unwrap();
    assert_eq!(
        compare_plays(&stronger_suit, &heaven, &strict),
        PlayComparison::Greater
    );
}
