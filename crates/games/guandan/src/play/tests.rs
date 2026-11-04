use super::{GuandanPlayKind, can_beat, classify};
use crate::{GuandanCard, GuandanSuit};

fn card(deck: u8, suit: GuandanSuit, rank: u8) -> GuandanCard {
    GuandanCard { deck, suit, rank }
}

#[test]
fn wild_heart_level_completes_a_straight_flush() {
    let cards = [
        card(0, GuandanSuit::Spades, 3),
        card(0, GuandanSuit::Spades, 4),
        card(0, GuandanSuit::Spades, 6),
        card(0, GuandanSuit::Spades, 7),
        card(0, GuandanSuit::Hearts, 9),
    ];
    let play = classify(&cards, 9).unwrap();
    assert_eq!(play.kind, GuandanPlayKind::StraightFlush);
    assert_eq!(play.value, 7);
}

#[test]
fn ace_low_straight_is_five_high() {
    let cards = [
        card(0, GuandanSuit::Spades, 14),
        card(0, GuandanSuit::Clubs, 2),
        card(0, GuandanSuit::Hearts, 3),
        card(0, GuandanSuit::Diamonds, 4),
        card(0, GuandanSuit::Spades, 5),
    ];
    let play = classify(&cards, 9).unwrap();
    assert_eq!(play.kind, GuandanPlayKind::Straight);
    assert_eq!(play.value, 5);
}

#[test]
fn bomb_order_places_straight_flush_between_five_and_six() {
    let five: Vec<_> = [
        card(0, GuandanSuit::Spades, 7),
        card(0, GuandanSuit::Hearts, 7),
        card(0, GuandanSuit::Clubs, 7),
        card(0, GuandanSuit::Diamonds, 7),
        card(1, GuandanSuit::Spades, 7),
    ]
    .into();
    let six: Vec<_> = [
        card(0, GuandanSuit::Spades, 8),
        card(0, GuandanSuit::Hearts, 8),
        card(0, GuandanSuit::Clubs, 8),
        card(0, GuandanSuit::Diamonds, 8),
        card(1, GuandanSuit::Spades, 8),
        card(1, GuandanSuit::Clubs, 8),
    ]
    .into();
    let flush: Vec<_> = [3, 4, 5, 6, 7]
        .into_iter()
        .map(|rank| card(0, GuandanSuit::Spades, rank))
        .collect();
    let five = classify(&five, 9).unwrap();
    let six = classify(&six, 9).unwrap();
    let flush = classify(&flush, 9).unwrap();
    assert!(can_beat(&flush, &five, 9));
    assert!(can_beat(&six, &flush, 9));
    assert!(!can_beat(&five, &flush, 9));
}

#[test]
fn level_rank_outweighs_ace_but_not_joker() {
    let level = classify(&[card(0, GuandanSuit::Clubs, 9)], 9).unwrap();
    let ace = classify(&[card(0, GuandanSuit::Spades, 14)], 9).unwrap();
    let joker = classify(&[card(0, GuandanSuit::Joker, 16)], 9).unwrap();
    assert!(can_beat(&level, &ace, 9));
    assert!(can_beat(&joker, &level, 9));
}

#[test]
fn both_heart_level_cards_can_play_as_a_level_pair() {
    let wild_pair = [
        card(0, GuandanSuit::Hearts, 9),
        card(1, GuandanSuit::Hearts, 9),
    ];
    let aces = [
        card(0, GuandanSuit::Spades, 14),
        card(1, GuandanSuit::Spades, 14),
    ];
    let wild_pair = classify(&wild_pair, 9).unwrap();
    assert_eq!(wild_pair.kind, GuandanPlayKind::Pair);
    assert_eq!(wild_pair.value, 9);
    assert!(can_beat(&wild_pair, &classify(&aces, 9).unwrap(), 9));
}

#[test]
fn joker_bomb_needs_all_four_jokers() {
    let cards = [
        card(0, GuandanSuit::Joker, 16),
        card(1, GuandanSuit::Joker, 16),
        card(0, GuandanSuit::Joker, 17),
        card(1, GuandanSuit::Joker, 17),
    ];
    assert_eq!(
        classify(&cards, 9).unwrap().kind,
        GuandanPlayKind::JokerBomb
    );
    assert!(classify(&cards[..3], 9).is_err());
}

#[test]
fn two_wild_cards_extend_an_eight_card_bomb_to_ten() {
    let mut cards = Vec::new();
    for deck in 0..2 {
        for suit in [
            GuandanSuit::Spades,
            GuandanSuit::Hearts,
            GuandanSuit::Clubs,
            GuandanSuit::Diamonds,
        ] {
            cards.push(card(deck, suit, 8));
        }
        cards.push(card(deck, GuandanSuit::Hearts, 9));
    }
    let play = classify(&cards, 9).unwrap();
    assert_eq!(play.kind, GuandanPlayKind::Bomb(10));
    let kings = [
        card(0, GuandanSuit::Joker, 16),
        card(1, GuandanSuit::Joker, 16),
        card(0, GuandanSuit::Joker, 17),
        card(1, GuandanSuit::Joker, 17),
    ];
    assert!(can_beat(&classify(&kings, 9).unwrap(), &play, 9));
}
