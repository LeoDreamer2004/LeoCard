use crate::ShengjiSuit;

use super::*;

fn pair(suit: crate::ShengjiSuit, rank: ShengjiRank) -> [ShengjiCard; 2] {
    [
        ShengjiCard::suited(0, suit, rank),
        ShengjiCard::suited(1, suit, rank),
    ]
}

#[test]
fn suit_counters_follow_diamond_club_heart_spade_order() {
    let diamond = pair(ShengjiSuit::Diamond, ShengjiRank::Ten);
    let heart = pair(ShengjiSuit::Heart, ShengjiRank::Ten);
    let club = pair(ShengjiSuit::Club, ShengjiRank::Ten);
    let hand = [diamond, heart, club].concat();
    let mut bidding = BidState::new(ShengjiRank::Ten);
    bidding
        .declare(ShengjiPlayerId(0), &diamond[..1], &hand)
        .unwrap();
    bidding.declare(ShengjiPlayerId(1), &heart, &hand).unwrap();
    assert_eq!(
        bidding.declare(ShengjiPlayerId(2), &club, &hand),
        Err(BidError::NotStronger)
    );
}

#[test]
fn self_protection_only_allows_no_trump_and_big_jokers_beat_small() {
    let diamond = pair(ShengjiSuit::Diamond, ShengjiRank::Ten);
    let spade = pair(ShengjiSuit::Spade, ShengjiRank::Ten);
    let small = [ShengjiCard::small_joker(0), ShengjiCard::small_joker(1)];
    let big = [ShengjiCard::big_joker(0), ShengjiCard::big_joker(1)];
    let hand = [diamond.as_slice(), spade.as_slice(), &small, &big].concat();
    let mut bidding = BidState::new(ShengjiRank::Ten);
    bidding
        .declare(ShengjiPlayerId(0), &diamond[..1], &hand)
        .unwrap();
    bidding
        .declare(ShengjiPlayerId(0), &diamond[1..], &hand)
        .unwrap();
    assert_eq!(
        bidding.declare(ShengjiPlayerId(1), &spade, &hand),
        Err(BidError::ProtectedSuit)
    );
    bidding.declare(ShengjiPlayerId(1), &small, &hand).unwrap();
    bidding.declare(ShengjiPlayerId(2), &big, &hand).unwrap();
    assert_eq!(
        bidding.current().unwrap().trump,
        ShengjiBidTrump::NoTrumpBigJoker
    );
}

#[test]
fn original_bidder_can_self_counter_with_another_suit_pair() {
    let diamond = pair(ShengjiSuit::Diamond, ShengjiRank::Ten);
    let heart = pair(ShengjiSuit::Heart, ShengjiRank::Ten);
    let spade = pair(ShengjiSuit::Spade, ShengjiRank::Ten);
    let hand = [diamond, heart, spade].concat();
    let mut bidding = BidState::new(ShengjiRank::Ten);
    bidding
        .declare(ShengjiPlayerId(0), &diamond[..1], &hand)
        .unwrap();
    bidding.declare(ShengjiPlayerId(1), &heart, &hand).unwrap();
    let declaration = bidding.declare(ShengjiPlayerId(0), &spade, &hand).unwrap();
    assert_eq!(declaration.kind, ShengjiBidKind::SelfCounter);
}

#[test]
fn three_deck_single_bids_cannot_counter_and_reinforcement_uses_more_copies() {
    let diamond = [
        ShengjiCard::suited(0, ShengjiSuit::Diamond, ShengjiRank::Ten),
        ShengjiCard::suited(1, ShengjiSuit::Diamond, ShengjiRank::Ten),
        ShengjiCard::suited(2, ShengjiSuit::Diamond, ShengjiRank::Ten),
    ];
    let heart_single = ShengjiCard::suited(0, ShengjiSuit::Heart, ShengjiRank::Ten);
    let hand = [diamond.as_slice(), &[heart_single]].concat();
    let mut bidding = BidState::new_with_decks(ShengjiRank::Ten, 3);

    bidding
        .declare(ShengjiPlayerId(0), &diamond[..1], &hand)
        .unwrap();
    assert_eq!(
        bidding.declare(ShengjiPlayerId(1), &[heart_single], &hand),
        Err(BidError::CounterRequiresPair)
    );
    let pair = bidding
        .declare(ShengjiPlayerId(0), &diamond[1..2], &hand)
        .unwrap();
    assert_eq!(pair.cards, diamond[..2]);
    let triple = bidding
        .declare(ShengjiPlayerId(0), &diamond[2..], &hand)
        .unwrap();
    assert_eq!(triple.cards, diamond);
}

#[test]
fn three_deck_bid_strength_uses_count_then_no_trump_and_suit_order() {
    let diamond_pair = ShengjiBidTrump::Suit(ShengjiSuit::Diamond)
        .three_deck_declaration_strength(2)
        .unwrap();
    let spade_pair = ShengjiBidTrump::Suit(ShengjiSuit::Spade)
        .three_deck_declaration_strength(2)
        .unwrap();
    let small_pair = ShengjiBidTrump::NoTrumpSmallJoker
        .three_deck_declaration_strength(2)
        .unwrap();
    let big_pair = ShengjiBidTrump::NoTrumpBigJoker
        .three_deck_declaration_strength(2)
        .unwrap();
    let diamond_triple = ShengjiBidTrump::Suit(ShengjiSuit::Diamond)
        .three_deck_declaration_strength(3)
        .unwrap();
    assert!(diamond_pair < spade_pair);
    assert!(spade_pair < small_pair);
    assert!(small_pair < big_pair);
    assert!(big_pair < diamond_triple);
}

#[test]
fn four_deck_bidding_extends_to_quad_level_and_quad_jokers() {
    let diamond = (0..4)
        .map(|deck| ShengjiCard::suited(deck, ShengjiSuit::Diamond, ShengjiRank::Ten))
        .collect::<Vec<_>>();
    let spade = (0..4)
        .map(|deck| ShengjiCard::suited(deck, ShengjiSuit::Spade, ShengjiRank::Ten))
        .collect::<Vec<_>>();
    let big = (0..4).map(ShengjiCard::big_joker).collect::<Vec<_>>();
    let hand = [diamond.as_slice(), spade.as_slice(), big.as_slice()].concat();
    let mut bidding = BidState::new_with_decks(ShengjiRank::Ten, 4);

    bidding
        .declare(ShengjiPlayerId(0), &diamond[..1], &hand)
        .unwrap();
    assert_eq!(
        bidding.declare(ShengjiPlayerId(1), &spade[..1], &hand),
        Err(BidError::CounterRequiresPair)
    );
    bidding
        .declare(ShengjiPlayerId(0), &diamond[1..2], &hand)
        .unwrap();
    bidding
        .declare(ShengjiPlayerId(0), &diamond[2..3], &hand)
        .unwrap();
    bidding
        .declare(ShengjiPlayerId(0), &diamond[3..], &hand)
        .unwrap();
    let declaration = bidding.declare(ShengjiPlayerId(2), &big, &hand).unwrap();
    assert_eq!(declaration.trump, ShengjiBidTrump::NoTrumpBigJoker);
    assert_eq!(declaration.cards.len(), 4);
}

#[test]
fn four_deck_strength_orders_count_before_no_trump_and_suit() {
    let big_triple = ShengjiBidTrump::NoTrumpBigJoker
        .declaration_strength(4, 3)
        .unwrap();
    let diamond_quad = ShengjiBidTrump::Suit(ShengjiSuit::Diamond)
        .declaration_strength(4, 4)
        .unwrap();
    let spade_quad = ShengjiBidTrump::Suit(ShengjiSuit::Spade)
        .declaration_strength(4, 4)
        .unwrap();
    let small_quad = ShengjiBidTrump::NoTrumpSmallJoker
        .declaration_strength(4, 4)
        .unwrap();
    let big_quad = ShengjiBidTrump::NoTrumpBigJoker
        .declaration_strength(4, 4)
        .unwrap();
    assert!(big_triple < diamond_quad);
    assert!(diamond_quad < spade_quad);
    assert!(spade_quad < small_quad);
    assert!(small_quad < big_quad);
}

#[test]
fn joker_bidding_requires_the_matching_color_and_cannot_open_no_trump() {
    let heart = ShengjiCard::suited(0, ShengjiSuit::Heart, ShengjiRank::Ten);
    let small = ShengjiCard::small_joker(0);
    let big = [ShengjiCard::big_joker(0), ShengjiCard::big_joker(1)];
    let hand = [heart, small, big[0], big[1]];
    let mut bidding = BidState::new_with_rules(ShengjiRank::Ten, 2, true);

    assert_eq!(
        bidding.declare(ShengjiPlayerId(0), &[heart], &hand),
        Err(BidError::JokerRequired)
    );
    assert_eq!(
        bidding.declare(ShengjiPlayerId(0), &[heart, small], &hand),
        Err(BidError::JokerRequired)
    );
    assert_eq!(
        bidding.declare(ShengjiPlayerId(0), &big, &hand),
        Err(BidError::NoTrumpCannotOpen)
    );
    let declaration = bidding
        .declare(ShengjiPlayerId(0), &[heart, big[0]], &hand)
        .unwrap();
    assert_eq!(declaration.trump, ShengjiBidTrump::Suit(ShengjiSuit::Heart));
}

#[test]
fn joker_bidding_reuses_the_owners_exposed_joker_for_protection_and_no_trump() {
    let heart = pair(ShengjiSuit::Heart, ShengjiRank::Ten);
    let big = [ShengjiCard::big_joker(0), ShengjiCard::big_joker(1)];
    let hand = [heart.as_slice(), big.as_slice()].concat();
    let mut bidding = BidState::new_with_rules(ShengjiRank::Ten, 2, true);

    bidding
        .declare(ShengjiPlayerId(0), &[heart[0], big[0]], &hand)
        .unwrap();
    assert_eq!(
        bidding.declare(ShengjiPlayerId(1), &big, &hand),
        Err(BidError::InvalidCards),
        "其它玩家不能借用已经亮出的王"
    );
    let protected = bidding
        .declare(ShengjiPlayerId(0), &[heart[1]], &hand)
        .unwrap();
    assert!(protected.protected);
    assert_eq!(protected.kind, ShengjiBidKind::Protect);
    assert_eq!(protected.cards, vec![heart[0], big[0], heart[1]]);

    let no_trump = bidding.declare(ShengjiPlayerId(0), &big, &hand).unwrap();
    assert_eq!(no_trump.trump, ShengjiBidTrump::NoTrumpBigJoker);
    assert_eq!(no_trump.cards, big);
}

#[test]
fn protected_joker_bid_ignores_suit_order_but_more_level_cards_can_counter() {
    let diamond = (0..3)
        .map(|deck| ShengjiCard::suited(deck, ShengjiSuit::Diamond, ShengjiRank::Ten))
        .collect::<Vec<_>>();
    let spade = (0..3)
        .map(|deck| ShengjiCard::suited(deck, ShengjiSuit::Spade, ShengjiRank::Ten))
        .collect::<Vec<_>>();
    let big = [ShengjiCard::big_joker(0), ShengjiCard::big_joker(1)];
    let small = [ShengjiCard::small_joker(0), ShengjiCard::small_joker(1)];
    let hand = [
        diamond.as_slice(),
        spade.as_slice(),
        big.as_slice(),
        small.as_slice(),
    ]
    .concat();

    let mut same_count = BidState::new_with_rules(ShengjiRank::Ten, 3, true);
    same_count
        .declare(ShengjiPlayerId(0), &[diamond[0], big[0]], &hand)
        .unwrap();
    same_count
        .declare(ShengjiPlayerId(0), &[diamond[1]], &hand)
        .unwrap();
    assert_eq!(
        same_count.declare(ShengjiPlayerId(1), &[spade[0], spade[1], small[0]], &hand),
        Err(BidError::ProtectedSuit)
    );

    let mut more_cards = BidState::new_with_rules(ShengjiRank::Ten, 3, true);
    more_cards
        .declare(ShengjiPlayerId(0), &[spade[0], small[0]], &hand)
        .unwrap();
    more_cards
        .declare(ShengjiPlayerId(0), &[spade[1]], &hand)
        .unwrap();
    let counter = more_cards
        .declare(
            ShengjiPlayerId(1),
            &[diamond[0], diamond[1], diamond[2], big[1]],
            &hand,
        )
        .unwrap();
    assert_eq!(counter.trump, ShengjiBidTrump::Suit(ShengjiSuit::Diamond));
}
