use super::super::candidates::plays;
use super::super::hand::counts;
use super::fixtures::{Table, card, play, trump};
use crate::{
    ShengjiCard, ShengjiRank, ShengjiSuit, ShengjiTrickPlay, classify_lead, validate_follow,
};
use std::collections::HashSet;

#[test]
fn equivalent_physical_copies_do_not_hide_high_winning_responses() {
    let mut hand: Vec<_> = (0..4)
        .flat_map(|deck| {
            [
                ShengjiRank::Two,
                ShengjiRank::Three,
                ShengjiRank::Four,
                ShengjiRank::Five,
            ]
            .map(move |rank| ShengjiCard::suited(deck, ShengjiSuit::Heart, rank))
        })
        .collect();
    hand.extend([ShengjiCard::big_joker(0), ShengjiCard::big_joker(1)]);
    let lead = play(&[
        card(ShengjiSuit::Heart, ShengjiRank::King),
        ShengjiCard::suited(1, ShengjiSuit::Heart, ShengjiRank::King),
    ]);
    let choices = plays(&hand, Some(&lead), trump(), 16);
    assert!(choices.iter().any(|play| {
        play.cards
            .iter()
            .all(|card| card.rank() == ShengjiRank::BigJoker)
    }));
    let distinct: HashSet<_> = choices.iter().map(|play| counts(&play.cards)).collect();
    assert_eq!(distinct.len(), choices.len());
    assert!(
        choices
            .iter()
            .all(|play| validate_follow(&hand, &play.cards, &lead, trump()).is_ok())
    );
}

#[test]
fn long_tractors_are_not_cut_at_six_pairs() {
    let hand: Vec<_> = ShengjiRank::LEVELS[..7]
        .iter()
        .flat_map(|rank| {
            (0..2).map(move |deck| ShengjiCard::suited(deck, ShengjiSuit::Spade, *rank))
        })
        .collect();
    assert!(
        plays(&hand, None, trump(), 16)
            .iter()
            .any(|play| play.cards.len() == 14 && !play.is_throw())
    );
}

#[test]
fn public_proof_allows_combined_throws_and_respects_the_rule_switch() {
    let ace = card(ShengjiSuit::Spade, ShengjiRank::Ace);
    let king = card(ShengjiSuit::Spade, ShengjiRank::King);
    let mut table = Table::new(vec![
        vec![ace, king],
        vec![card(ShengjiSuit::Spade, ShengjiRank::Three)],
        vec![],
        vec![],
    ]);
    let chosen = table.best(0);
    assert!(chosen.is_throw());
    let others: Vec<_> = table.hands[1..].iter().map(Vec::as_slice).collect();
    assert!(matches!(
        classify_lead(&chosen.cards, table.trump, &table.rules, &others),
        Ok(ShengjiTrickPlay::Accepted(_))
    ));
    table.rules.allow_throw = false;
    assert!(!table.best(0).is_throw());
}
