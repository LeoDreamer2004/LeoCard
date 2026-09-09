use super::*;
use crate::TexasHoldemSuit::{Club, Diamond, Heart, Spade};

fn c(rank: TexasHoldemRank, suit: crate::TexasHoldemSuit) -> TexasHoldemCard {
    TexasHoldemCard::new(suit, rank)
}

fn rules(short_deck: bool) -> TexasHoldemRuleSet {
    TexasHoldemRuleSet {
        short_deck,
        ..TexasHoldemRuleSet::default()
    }
}

#[test]
fn standard_category_order_matches_holdem() {
    let flush = evaluate_best(
        &[
            c(TexasHoldemRank::Ace, Heart),
            c(TexasHoldemRank::Jack, Heart),
            c(TexasHoldemRank::Nine, Heart),
            c(TexasHoldemRank::Seven, Heart),
            c(TexasHoldemRank::Three, Heart),
        ],
        &rules(false),
    )
    .unwrap();
    let full_house = evaluate_best(
        &[
            c(TexasHoldemRank::King, Spade),
            c(TexasHoldemRank::King, Heart),
            c(TexasHoldemRank::King, Club),
            c(TexasHoldemRank::Nine, Spade),
            c(TexasHoldemRank::Nine, Diamond),
        ],
        &rules(false),
    )
    .unwrap();
    let straight = evaluate_best(
        &[
            c(TexasHoldemRank::Nine, Spade),
            c(TexasHoldemRank::Eight, Heart),
            c(TexasHoldemRank::Seven, Club),
            c(TexasHoldemRank::Six, Diamond),
            c(TexasHoldemRank::Five, Heart),
        ],
        &rules(false),
    )
    .unwrap();
    let trips = evaluate_best(
        &[
            c(TexasHoldemRank::Ace, Spade),
            c(TexasHoldemRank::Ace, Heart),
            c(TexasHoldemRank::Ace, Club),
            c(TexasHoldemRank::King, Diamond),
            c(TexasHoldemRank::Queen, Heart),
        ],
        &rules(false),
    )
    .unwrap();
    assert!(full_house > flush);
    assert!(flush > straight);
    assert!(straight > trips);
}

#[test]
fn short_deck_moves_flush_and_trips_above_full_house_and_straight() {
    let short = rules(true);
    let flush = evaluate_best(
        &[
            c(TexasHoldemRank::Ace, Heart),
            c(TexasHoldemRank::Jack, Heart),
            c(TexasHoldemRank::Nine, Heart),
            c(TexasHoldemRank::Seven, Heart),
            c(TexasHoldemRank::Six, Heart),
        ],
        &short,
    )
    .unwrap();
    let full_house = evaluate_best(
        &[
            c(TexasHoldemRank::King, Spade),
            c(TexasHoldemRank::King, Heart),
            c(TexasHoldemRank::King, Club),
            c(TexasHoldemRank::Nine, Spade),
            c(TexasHoldemRank::Nine, Diamond),
        ],
        &short,
    )
    .unwrap();
    let trips = evaluate_best(
        &[
            c(TexasHoldemRank::Ace, Spade),
            c(TexasHoldemRank::Ace, Heart),
            c(TexasHoldemRank::Ace, Club),
            c(TexasHoldemRank::King, Diamond),
            c(TexasHoldemRank::Queen, Heart),
        ],
        &short,
    )
    .unwrap();
    let straight = evaluate_best(
        &[
            c(TexasHoldemRank::Ten, Spade),
            c(TexasHoldemRank::Nine, Heart),
            c(TexasHoldemRank::Eight, Club),
            c(TexasHoldemRank::Seven, Diamond),
            c(TexasHoldemRank::Six, Heart),
        ],
        &short,
    )
    .unwrap();
    assert!(flush > full_house);
    assert!(trips > straight);
}

#[test]
fn royal_flush_is_distinct_and_seven_cards_choose_the_best_five() {
    let hand = evaluate_best(
        &[
            c(TexasHoldemRank::Ace, Spade),
            c(TexasHoldemRank::King, Spade),
            c(TexasHoldemRank::Queen, Spade),
            c(TexasHoldemRank::Jack, Spade),
            c(TexasHoldemRank::Ten, Spade),
            c(TexasHoldemRank::Two, Club),
            c(TexasHoldemRank::Two, Diamond),
        ],
        &rules(false),
    )
    .unwrap();
    assert_eq!(hand.category(), TexasHoldemHandCategory::RoyalFlush);
}

#[test]
fn ace_can_be_low_in_each_deck_variant() {
    let standard = evaluate_best(
        &[
            c(TexasHoldemRank::Ace, Spade),
            c(TexasHoldemRank::Five, Heart),
            c(TexasHoldemRank::Four, Club),
            c(TexasHoldemRank::Three, Diamond),
            c(TexasHoldemRank::Two, Heart),
        ],
        &rules(false),
    )
    .unwrap();
    let short = evaluate_best(
        &[
            c(TexasHoldemRank::Ace, Spade),
            c(TexasHoldemRank::Nine, Heart),
            c(TexasHoldemRank::Eight, Club),
            c(TexasHoldemRank::Seven, Diamond),
            c(TexasHoldemRank::Six, Heart),
        ],
        &rules(true),
    )
    .unwrap();
    assert_eq!(standard.kickers()[0], 5);
    assert_eq!(short.kickers()[0], 9);

    let standard_six_high = evaluate_best(
        &[
            c(TexasHoldemRank::Six, Spade),
            c(TexasHoldemRank::Five, Diamond),
            c(TexasHoldemRank::Four, Heart),
            c(TexasHoldemRank::Three, Club),
            c(TexasHoldemRank::Two, Spade),
        ],
        &rules(false),
    )
    .unwrap();
    let short_ten_high = evaluate_best(
        &[
            c(TexasHoldemRank::Ten, Spade),
            c(TexasHoldemRank::Nine, Diamond),
            c(TexasHoldemRank::Eight, Heart),
            c(TexasHoldemRank::Seven, Club),
            c(TexasHoldemRank::Six, Spade),
        ],
        &rules(true),
    )
    .unwrap();
    assert!(standard < standard_six_high, "A2345 必须是普通德州最小顺子");
    assert!(short < short_ten_high, "A6789 必须是短牌德州最小顺子");
}

#[test]
fn selected_five_cards_have_poker_readable_display_order() {
    let quads = evaluate_best(
        &[
            c(TexasHoldemRank::Ace, Diamond),
            c(TexasHoldemRank::Nine, Club),
            c(TexasHoldemRank::Nine, Spade),
            c(TexasHoldemRank::Nine, Heart),
            c(TexasHoldemRank::Nine, Diamond),
        ],
        &rules(false),
    )
    .unwrap();
    assert_eq!(
        quads.cards().map(TexasHoldemCard::rank),
        [
            TexasHoldemRank::Nine,
            TexasHoldemRank::Nine,
            TexasHoldemRank::Nine,
            TexasHoldemRank::Nine,
            TexasHoldemRank::Ace
        ]
    );

    let full_house = evaluate_best(
        &[
            c(TexasHoldemRank::Ace, Spade),
            c(TexasHoldemRank::Two, Club),
            c(TexasHoldemRank::Ace, Heart),
            c(TexasHoldemRank::Two, Spade),
            c(TexasHoldemRank::Two, Diamond),
        ],
        &rules(false),
    )
    .unwrap();
    assert_eq!(
        full_house.cards().map(TexasHoldemCard::rank),
        [
            TexasHoldemRank::Two,
            TexasHoldemRank::Two,
            TexasHoldemRank::Two,
            TexasHoldemRank::Ace,
            TexasHoldemRank::Ace
        ]
    );

    let pair = evaluate_best(
        &[
            c(TexasHoldemRank::Queen, Diamond),
            c(TexasHoldemRank::Six, Spade),
            c(TexasHoldemRank::Ace, Heart),
            c(TexasHoldemRank::Six, Club),
            c(TexasHoldemRank::King, Diamond),
        ],
        &rules(false),
    )
    .unwrap();
    assert_eq!(
        pair.cards().map(TexasHoldemCard::rank),
        [
            TexasHoldemRank::Six,
            TexasHoldemRank::Six,
            TexasHoldemRank::Ace,
            TexasHoldemRank::King,
            TexasHoldemRank::Queen
        ]
    );

    let high_card = evaluate_best(
        &[
            c(TexasHoldemRank::Nine, Spade),
            c(TexasHoldemRank::Queen, Club),
            c(TexasHoldemRank::Ace, Diamond),
            c(TexasHoldemRank::Jack, Heart),
            c(TexasHoldemRank::King, Spade),
        ],
        &rules(false),
    )
    .unwrap();
    assert_eq!(
        high_card.cards().map(TexasHoldemCard::rank),
        [
            TexasHoldemRank::Ace,
            TexasHoldemRank::King,
            TexasHoldemRank::Queen,
            TexasHoldemRank::Jack,
            TexasHoldemRank::Nine
        ]
    );

    let wheel = evaluate_best(
        &[
            c(TexasHoldemRank::Ace, Spade),
            c(TexasHoldemRank::Three, Club),
            c(TexasHoldemRank::Five, Heart),
            c(TexasHoldemRank::Two, Diamond),
            c(TexasHoldemRank::Four, Spade),
        ],
        &rules(false),
    )
    .unwrap();
    assert_eq!(
        wheel.cards().map(TexasHoldemCard::rank),
        [
            TexasHoldemRank::Five,
            TexasHoldemRank::Four,
            TexasHoldemRank::Three,
            TexasHoldemRank::Two,
            TexasHoldemRank::Ace
        ]
    );

    let short_wheel = evaluate_best(
        &[
            c(TexasHoldemRank::Ace, Spade),
            c(TexasHoldemRank::Seven, Club),
            c(TexasHoldemRank::Nine, Heart),
            c(TexasHoldemRank::Six, Diamond),
            c(TexasHoldemRank::Eight, Spade),
        ],
        &rules(true),
    )
    .unwrap();
    assert_eq!(
        short_wheel.cards().map(TexasHoldemCard::rank),
        [
            TexasHoldemRank::Nine,
            TexasHoldemRank::Eight,
            TexasHoldemRank::Seven,
            TexasHoldemRank::Six,
            TexasHoldemRank::Ace
        ]
    );
}

#[test]
fn every_declared_standard_category_is_in_exact_order() {
    let hands = [
        [
            c(TexasHoldemRank::Ace, Spade),
            c(TexasHoldemRank::King, Spade),
            c(TexasHoldemRank::Queen, Spade),
            c(TexasHoldemRank::Jack, Spade),
            c(TexasHoldemRank::Ten, Spade),
        ],
        [
            c(TexasHoldemRank::Nine, Heart),
            c(TexasHoldemRank::Eight, Heart),
            c(TexasHoldemRank::Seven, Heart),
            c(TexasHoldemRank::Six, Heart),
            c(TexasHoldemRank::Five, Heart),
        ],
        [
            c(TexasHoldemRank::Ace, Spade),
            c(TexasHoldemRank::Ace, Heart),
            c(TexasHoldemRank::Ace, Club),
            c(TexasHoldemRank::Ace, Diamond),
            c(TexasHoldemRank::King, Spade),
        ],
        [
            c(TexasHoldemRank::King, Spade),
            c(TexasHoldemRank::King, Heart),
            c(TexasHoldemRank::King, Club),
            c(TexasHoldemRank::Queen, Spade),
            c(TexasHoldemRank::Queen, Heart),
        ],
        [
            c(TexasHoldemRank::Ace, Club),
            c(TexasHoldemRank::Jack, Club),
            c(TexasHoldemRank::Nine, Club),
            c(TexasHoldemRank::Seven, Club),
            c(TexasHoldemRank::Three, Club),
        ],
        [
            c(TexasHoldemRank::Ten, Spade),
            c(TexasHoldemRank::Nine, Heart),
            c(TexasHoldemRank::Eight, Club),
            c(TexasHoldemRank::Seven, Diamond),
            c(TexasHoldemRank::Six, Spade),
        ],
        [
            c(TexasHoldemRank::Jack, Spade),
            c(TexasHoldemRank::Jack, Heart),
            c(TexasHoldemRank::Jack, Club),
            c(TexasHoldemRank::Ace, Diamond),
            c(TexasHoldemRank::King, Spade),
        ],
        [
            c(TexasHoldemRank::Ace, Spade),
            c(TexasHoldemRank::Ace, Heart),
            c(TexasHoldemRank::King, Club),
            c(TexasHoldemRank::King, Diamond),
            c(TexasHoldemRank::Queen, Spade),
        ],
        [
            c(TexasHoldemRank::Ace, Spade),
            c(TexasHoldemRank::Ace, Heart),
            c(TexasHoldemRank::King, Club),
            c(TexasHoldemRank::Queen, Diamond),
            c(TexasHoldemRank::Jack, Spade),
        ],
        [
            c(TexasHoldemRank::Ace, Spade),
            c(TexasHoldemRank::King, Heart),
            c(TexasHoldemRank::Queen, Club),
            c(TexasHoldemRank::Jack, Diamond),
            c(TexasHoldemRank::Nine, Spade),
        ],
    ]
    .map(|cards| evaluate_best(&cards, &rules(false)).unwrap());
    assert!(hands.windows(2).all(|pair| pair[0] > pair[1]));
    assert_eq!(
        hands.map(EvaluatedHand::category),
        [
            TexasHoldemHandCategory::RoyalFlush,
            TexasHoldemHandCategory::StraightFlush,
            TexasHoldemHandCategory::FourOfAKind,
            TexasHoldemHandCategory::FullHouse,
            TexasHoldemHandCategory::Flush,
            TexasHoldemHandCategory::Straight,
            TexasHoldemHandCategory::ThreeOfAKind,
            TexasHoldemHandCategory::TwoPair,
            TexasHoldemHandCategory::OnePair,
            TexasHoldemHandCategory::HighCard,
        ]
    );
}

#[test]
fn equal_rank_hands_split_regardless_of_suit() {
    let left = evaluate_best(
        &[
            c(TexasHoldemRank::Ace, Spade),
            c(TexasHoldemRank::Ace, Heart),
            c(TexasHoldemRank::King, Club),
            c(TexasHoldemRank::Queen, Diamond),
            c(TexasHoldemRank::Jack, Spade),
        ],
        &rules(false),
    )
    .unwrap();
    let right = evaluate_best(
        &[
            c(TexasHoldemRank::Ace, Club),
            c(TexasHoldemRank::Ace, Diamond),
            c(TexasHoldemRank::King, Heart),
            c(TexasHoldemRank::Queen, Spade),
            c(TexasHoldemRank::Jack, Club),
        ],
        &rules(false),
    )
    .unwrap();
    assert_eq!(left, right);
}

#[test]
fn ignore_kickers_compares_only_the_made_hand_and_largest_high_card() {
    let standard = rules(false);
    let ignore_kickers = TexasHoldemRuleSet {
        ignore_kickers: true,
        ..standard
    };
    let ace_queen_with_king = evaluate_best(
        &[
            c(TexasHoldemRank::Ace, Spade),
            c(TexasHoldemRank::Ace, Heart),
            c(TexasHoldemRank::Queen, Club),
            c(TexasHoldemRank::Queen, Diamond),
            c(TexasHoldemRank::King, Spade),
        ],
        &standard,
    )
    .unwrap();
    let ace_queen_with_jack = evaluate_best(
        &[
            c(TexasHoldemRank::Ace, Club),
            c(TexasHoldemRank::Ace, Diamond),
            c(TexasHoldemRank::Queen, Spade),
            c(TexasHoldemRank::Queen, Heart),
            c(TexasHoldemRank::Jack, Club),
        ],
        &standard,
    )
    .unwrap();
    assert!(ace_queen_with_king > ace_queen_with_jack);
    assert_eq!(
        ace_queen_with_king.cmp_with_rules(&ace_queen_with_jack, &ignore_kickers),
        Ordering::Equal
    );

    let ace_high = evaluate_best(
        &[
            c(TexasHoldemRank::Ace, Spade),
            c(TexasHoldemRank::Eight, Heart),
            c(TexasHoldemRank::Seven, Club),
            c(TexasHoldemRank::Five, Diamond),
            c(TexasHoldemRank::Three, Spade),
        ],
        &standard,
    )
    .unwrap();
    let same_ace_high = evaluate_best(
        &[
            c(TexasHoldemRank::Ace, Club),
            c(TexasHoldemRank::King, Diamond),
            c(TexasHoldemRank::Queen, Spade),
            c(TexasHoldemRank::Nine, Heart),
            c(TexasHoldemRank::Two, Club),
        ],
        &standard,
    )
    .unwrap();
    let king_high = evaluate_best(
        &[
            c(TexasHoldemRank::King, Spade),
            c(TexasHoldemRank::Queen, Heart),
            c(TexasHoldemRank::Jack, Club),
            c(TexasHoldemRank::Nine, Diamond),
            c(TexasHoldemRank::Seven, Spade),
        ],
        &standard,
    )
    .unwrap();
    assert_eq!(
        ace_high.cmp_with_rules(&same_ace_high, &ignore_kickers),
        Ordering::Equal
    );
    assert_eq!(
        ace_high.cmp_with_rules(&king_high, &ignore_kickers),
        Ordering::Greater
    );
}

#[test]
fn omaha_uses_exactly_two_hole_cards_and_three_board_cards() {
    let omaha = TexasHoldemRuleSet {
        omaha: true,
        ..TexasHoldemRuleSet::default()
    };
    let board = [
        c(TexasHoldemRank::Ace, Heart),
        c(TexasHoldemRank::King, Heart),
        c(TexasHoldemRank::Queen, Heart),
        c(TexasHoldemRank::Jack, Heart),
        c(TexasHoldemRank::Ten, Heart),
    ];
    let hand = evaluate_player_hand(
        &[
            c(TexasHoldemRank::Ace, Spade),
            c(TexasHoldemRank::Ace, Diamond),
            c(TexasHoldemRank::Four, Club),
            c(TexasHoldemRank::Five, Club),
        ],
        &board,
        &omaha,
    )
    .unwrap();

    assert_eq!(hand.category(), TexasHoldemHandCategory::ThreeOfAKind);
    assert_eq!(
        hand.cards()
            .iter()
            .filter(|card| board.contains(card))
            .count(),
        3
    );
}

#[test]
fn omaha_can_form_a_hand_from_two_hole_and_three_board_cards() {
    let omaha = TexasHoldemRuleSet {
        omaha: true,
        ..TexasHoldemRuleSet::default()
    };
    let hand = evaluate_omaha(
        &[
            c(TexasHoldemRank::Ace, Heart),
            c(TexasHoldemRank::King, Heart),
            c(TexasHoldemRank::Four, Club),
            c(TexasHoldemRank::Five, Club),
        ],
        &[
            c(TexasHoldemRank::Queen, Heart),
            c(TexasHoldemRank::Jack, Heart),
            c(TexasHoldemRank::Ten, Heart),
            c(TexasHoldemRank::Two, Spade),
            c(TexasHoldemRank::Three, Diamond),
        ],
        &omaha,
    )
    .unwrap();

    assert_eq!(hand.category(), TexasHoldemHandCategory::RoyalFlush);
}
