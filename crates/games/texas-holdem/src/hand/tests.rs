use super::*;
use crate::TexasSuit::{Club, Diamond, Heart, Spade};
use crate::{TexasCard, TexasRank, TexasRuleSet};
use std::cmp::Ordering;

fn c(rank: TexasRank, suit: crate::TexasSuit) -> TexasCard {
    TexasCard::new(suit, rank)
}

fn rules(short_deck: bool) -> TexasRuleSet {
    TexasRuleSet {
        short_deck,
        ..TexasRuleSet::default()
    }
}

#[test]
fn standard_category_order_matches_holdem() {
    let flush = evaluate_best(
        &[
            c(TexasRank::Ace, Heart),
            c(TexasRank::Jack, Heart),
            c(TexasRank::Nine, Heart),
            c(TexasRank::Seven, Heart),
            c(TexasRank::Three, Heart),
        ],
        &rules(false),
    )
    .unwrap();
    let full_house = evaluate_best(
        &[
            c(TexasRank::King, Spade),
            c(TexasRank::King, Heart),
            c(TexasRank::King, Club),
            c(TexasRank::Nine, Spade),
            c(TexasRank::Nine, Diamond),
        ],
        &rules(false),
    )
    .unwrap();
    let straight = evaluate_best(
        &[
            c(TexasRank::Nine, Spade),
            c(TexasRank::Eight, Heart),
            c(TexasRank::Seven, Club),
            c(TexasRank::Six, Diamond),
            c(TexasRank::Five, Heart),
        ],
        &rules(false),
    )
    .unwrap();
    let trips = evaluate_best(
        &[
            c(TexasRank::Ace, Spade),
            c(TexasRank::Ace, Heart),
            c(TexasRank::Ace, Club),
            c(TexasRank::King, Diamond),
            c(TexasRank::Queen, Heart),
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
            c(TexasRank::Ace, Heart),
            c(TexasRank::Jack, Heart),
            c(TexasRank::Nine, Heart),
            c(TexasRank::Seven, Heart),
            c(TexasRank::Six, Heart),
        ],
        &short,
    )
    .unwrap();
    let full_house = evaluate_best(
        &[
            c(TexasRank::King, Spade),
            c(TexasRank::King, Heart),
            c(TexasRank::King, Club),
            c(TexasRank::Nine, Spade),
            c(TexasRank::Nine, Diamond),
        ],
        &short,
    )
    .unwrap();
    let trips = evaluate_best(
        &[
            c(TexasRank::Ace, Spade),
            c(TexasRank::Ace, Heart),
            c(TexasRank::Ace, Club),
            c(TexasRank::King, Diamond),
            c(TexasRank::Queen, Heart),
        ],
        &short,
    )
    .unwrap();
    let straight = evaluate_best(
        &[
            c(TexasRank::Ten, Spade),
            c(TexasRank::Nine, Heart),
            c(TexasRank::Eight, Club),
            c(TexasRank::Seven, Diamond),
            c(TexasRank::Six, Heart),
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
            c(TexasRank::Ace, Spade),
            c(TexasRank::King, Spade),
            c(TexasRank::Queen, Spade),
            c(TexasRank::Jack, Spade),
            c(TexasRank::Ten, Spade),
            c(TexasRank::Two, Club),
            c(TexasRank::Two, Diamond),
        ],
        &rules(false),
    )
    .unwrap();
    assert_eq!(hand.category(), TexasHandCategory::RoyalFlush);
}

#[test]
fn ace_can_be_low_in_each_deck_variant() {
    let standard = evaluate_best(
        &[
            c(TexasRank::Ace, Spade),
            c(TexasRank::Five, Heart),
            c(TexasRank::Four, Club),
            c(TexasRank::Three, Diamond),
            c(TexasRank::Two, Heart),
        ],
        &rules(false),
    )
    .unwrap();
    let short = evaluate_best(
        &[
            c(TexasRank::Ace, Spade),
            c(TexasRank::Nine, Heart),
            c(TexasRank::Eight, Club),
            c(TexasRank::Seven, Diamond),
            c(TexasRank::Six, Heart),
        ],
        &rules(true),
    )
    .unwrap();
    assert_eq!(standard.kickers()[0], 5);
    assert_eq!(short.kickers()[0], 9);

    let standard_six_high = evaluate_best(
        &[
            c(TexasRank::Six, Spade),
            c(TexasRank::Five, Diamond),
            c(TexasRank::Four, Heart),
            c(TexasRank::Three, Club),
            c(TexasRank::Two, Spade),
        ],
        &rules(false),
    )
    .unwrap();
    let short_ten_high = evaluate_best(
        &[
            c(TexasRank::Ten, Spade),
            c(TexasRank::Nine, Diamond),
            c(TexasRank::Eight, Heart),
            c(TexasRank::Seven, Club),
            c(TexasRank::Six, Spade),
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
            c(TexasRank::Ace, Diamond),
            c(TexasRank::Nine, Club),
            c(TexasRank::Nine, Spade),
            c(TexasRank::Nine, Heart),
            c(TexasRank::Nine, Diamond),
        ],
        &rules(false),
    )
    .unwrap();
    assert_eq!(
        quads.cards().map(TexasCard::rank),
        [
            TexasRank::Nine,
            TexasRank::Nine,
            TexasRank::Nine,
            TexasRank::Nine,
            TexasRank::Ace
        ]
    );

    let full_house = evaluate_best(
        &[
            c(TexasRank::Ace, Spade),
            c(TexasRank::Two, Club),
            c(TexasRank::Ace, Heart),
            c(TexasRank::Two, Spade),
            c(TexasRank::Two, Diamond),
        ],
        &rules(false),
    )
    .unwrap();
    assert_eq!(
        full_house.cards().map(TexasCard::rank),
        [
            TexasRank::Two,
            TexasRank::Two,
            TexasRank::Two,
            TexasRank::Ace,
            TexasRank::Ace
        ]
    );

    let pair = evaluate_best(
        &[
            c(TexasRank::Queen, Diamond),
            c(TexasRank::Six, Spade),
            c(TexasRank::Ace, Heart),
            c(TexasRank::Six, Club),
            c(TexasRank::King, Diamond),
        ],
        &rules(false),
    )
    .unwrap();
    assert_eq!(
        pair.cards().map(TexasCard::rank),
        [
            TexasRank::Six,
            TexasRank::Six,
            TexasRank::Ace,
            TexasRank::King,
            TexasRank::Queen
        ]
    );

    let high_card = evaluate_best(
        &[
            c(TexasRank::Nine, Spade),
            c(TexasRank::Queen, Club),
            c(TexasRank::Ace, Diamond),
            c(TexasRank::Jack, Heart),
            c(TexasRank::King, Spade),
        ],
        &rules(false),
    )
    .unwrap();
    assert_eq!(
        high_card.cards().map(TexasCard::rank),
        [
            TexasRank::Ace,
            TexasRank::King,
            TexasRank::Queen,
            TexasRank::Jack,
            TexasRank::Nine
        ]
    );

    let wheel = evaluate_best(
        &[
            c(TexasRank::Ace, Spade),
            c(TexasRank::Three, Club),
            c(TexasRank::Five, Heart),
            c(TexasRank::Two, Diamond),
            c(TexasRank::Four, Spade),
        ],
        &rules(false),
    )
    .unwrap();
    assert_eq!(
        wheel.cards().map(TexasCard::rank),
        [
            TexasRank::Five,
            TexasRank::Four,
            TexasRank::Three,
            TexasRank::Two,
            TexasRank::Ace
        ]
    );

    let short_wheel = evaluate_best(
        &[
            c(TexasRank::Ace, Spade),
            c(TexasRank::Seven, Club),
            c(TexasRank::Nine, Heart),
            c(TexasRank::Six, Diamond),
            c(TexasRank::Eight, Spade),
        ],
        &rules(true),
    )
    .unwrap();
    assert_eq!(
        short_wheel.cards().map(TexasCard::rank),
        [
            TexasRank::Nine,
            TexasRank::Eight,
            TexasRank::Seven,
            TexasRank::Six,
            TexasRank::Ace
        ]
    );
}

#[test]
fn every_declared_standard_category_is_in_exact_order() {
    let hands = [
        [
            c(TexasRank::Ace, Spade),
            c(TexasRank::King, Spade),
            c(TexasRank::Queen, Spade),
            c(TexasRank::Jack, Spade),
            c(TexasRank::Ten, Spade),
        ],
        [
            c(TexasRank::Nine, Heart),
            c(TexasRank::Eight, Heart),
            c(TexasRank::Seven, Heart),
            c(TexasRank::Six, Heart),
            c(TexasRank::Five, Heart),
        ],
        [
            c(TexasRank::Ace, Spade),
            c(TexasRank::Ace, Heart),
            c(TexasRank::Ace, Club),
            c(TexasRank::Ace, Diamond),
            c(TexasRank::King, Spade),
        ],
        [
            c(TexasRank::King, Spade),
            c(TexasRank::King, Heart),
            c(TexasRank::King, Club),
            c(TexasRank::Queen, Spade),
            c(TexasRank::Queen, Heart),
        ],
        [
            c(TexasRank::Ace, Club),
            c(TexasRank::Jack, Club),
            c(TexasRank::Nine, Club),
            c(TexasRank::Seven, Club),
            c(TexasRank::Three, Club),
        ],
        [
            c(TexasRank::Ten, Spade),
            c(TexasRank::Nine, Heart),
            c(TexasRank::Eight, Club),
            c(TexasRank::Seven, Diamond),
            c(TexasRank::Six, Spade),
        ],
        [
            c(TexasRank::Jack, Spade),
            c(TexasRank::Jack, Heart),
            c(TexasRank::Jack, Club),
            c(TexasRank::Ace, Diamond),
            c(TexasRank::King, Spade),
        ],
        [
            c(TexasRank::Ace, Spade),
            c(TexasRank::Ace, Heart),
            c(TexasRank::King, Club),
            c(TexasRank::King, Diamond),
            c(TexasRank::Queen, Spade),
        ],
        [
            c(TexasRank::Ace, Spade),
            c(TexasRank::Ace, Heart),
            c(TexasRank::King, Club),
            c(TexasRank::Queen, Diamond),
            c(TexasRank::Jack, Spade),
        ],
        [
            c(TexasRank::Ace, Spade),
            c(TexasRank::King, Heart),
            c(TexasRank::Queen, Club),
            c(TexasRank::Jack, Diamond),
            c(TexasRank::Nine, Spade),
        ],
    ]
    .map(|cards| evaluate_best(&cards, &rules(false)).unwrap());
    assert!(hands.windows(2).all(|pair| pair[0] > pair[1]));
    assert_eq!(
        hands.map(TexasEvaluatedHand::category),
        [
            TexasHandCategory::RoyalFlush,
            TexasHandCategory::StraightFlush,
            TexasHandCategory::FourOfAKind,
            TexasHandCategory::FullHouse,
            TexasHandCategory::Flush,
            TexasHandCategory::Straight,
            TexasHandCategory::ThreeOfAKind,
            TexasHandCategory::TwoPair,
            TexasHandCategory::OnePair,
            TexasHandCategory::HighCard,
        ]
    );
}

#[test]
fn equal_rank_hands_split_regardless_of_suit() {
    let left = evaluate_best(
        &[
            c(TexasRank::Ace, Spade),
            c(TexasRank::Ace, Heart),
            c(TexasRank::King, Club),
            c(TexasRank::Queen, Diamond),
            c(TexasRank::Jack, Spade),
        ],
        &rules(false),
    )
    .unwrap();
    let right = evaluate_best(
        &[
            c(TexasRank::Ace, Club),
            c(TexasRank::Ace, Diamond),
            c(TexasRank::King, Heart),
            c(TexasRank::Queen, Spade),
            c(TexasRank::Jack, Club),
        ],
        &rules(false),
    )
    .unwrap();
    assert_eq!(left, right);
}

#[test]
fn ignore_kickers_compares_only_the_made_hand_and_largest_high_card() {
    let standard = rules(false);
    let ignore_kickers = TexasRuleSet {
        ignore_kickers: true,
        ..standard
    };
    let ace_queen_with_king = evaluate_best(
        &[
            c(TexasRank::Ace, Spade),
            c(TexasRank::Ace, Heart),
            c(TexasRank::Queen, Club),
            c(TexasRank::Queen, Diamond),
            c(TexasRank::King, Spade),
        ],
        &standard,
    )
    .unwrap();
    let ace_queen_with_jack = evaluate_best(
        &[
            c(TexasRank::Ace, Club),
            c(TexasRank::Ace, Diamond),
            c(TexasRank::Queen, Spade),
            c(TexasRank::Queen, Heart),
            c(TexasRank::Jack, Club),
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
            c(TexasRank::Ace, Spade),
            c(TexasRank::Eight, Heart),
            c(TexasRank::Seven, Club),
            c(TexasRank::Five, Diamond),
            c(TexasRank::Three, Spade),
        ],
        &standard,
    )
    .unwrap();
    let same_ace_high = evaluate_best(
        &[
            c(TexasRank::Ace, Club),
            c(TexasRank::King, Diamond),
            c(TexasRank::Queen, Spade),
            c(TexasRank::Nine, Heart),
            c(TexasRank::Two, Club),
        ],
        &standard,
    )
    .unwrap();
    let king_high = evaluate_best(
        &[
            c(TexasRank::King, Spade),
            c(TexasRank::Queen, Heart),
            c(TexasRank::Jack, Club),
            c(TexasRank::Nine, Diamond),
            c(TexasRank::Seven, Spade),
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
    let omaha = TexasRuleSet {
        omaha: true,
        ..TexasRuleSet::default()
    };
    let board = [
        c(TexasRank::Ace, Heart),
        c(TexasRank::King, Heart),
        c(TexasRank::Queen, Heart),
        c(TexasRank::Jack, Heart),
        c(TexasRank::Ten, Heart),
    ];
    let hand = evaluate_player_hand(
        &[
            c(TexasRank::Ace, Spade),
            c(TexasRank::Ace, Diamond),
            c(TexasRank::Four, Club),
            c(TexasRank::Five, Club),
        ],
        &board,
        &omaha,
    )
    .unwrap();

    assert_eq!(hand.category(), TexasHandCategory::ThreeOfAKind);
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
    let omaha = TexasRuleSet {
        omaha: true,
        ..TexasRuleSet::default()
    };
    let hand = evaluate_omaha(
        &[
            c(TexasRank::Ace, Heart),
            c(TexasRank::King, Heart),
            c(TexasRank::Four, Club),
            c(TexasRank::Five, Club),
        ],
        &[
            c(TexasRank::Queen, Heart),
            c(TexasRank::Jack, Heart),
            c(TexasRank::Ten, Heart),
            c(TexasRank::Two, Spade),
            c(TexasRank::Three, Diamond),
        ],
        &omaha,
    )
    .unwrap();

    assert_eq!(hand.category(), TexasHandCategory::RoyalFlush);
}
