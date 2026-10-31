use leocard_game_common::PlayerId;
use leocard_shengji::{
    ShengjiBiddingObservation, ShengjiBot, ShengjiCard, ShengjiGameState, ShengjiRank,
    ShengjiRuleSet, ShengjiSuit, ShengjiTeamProgress, build_deck,
};

fn game_with_hand(
    player: PlayerId,
    jokers: usize,
    spades: usize,
    bottom_copy: bool,
) -> ShengjiGameState {
    let mut hand = vec![ShengjiCard::small_joker(0), ShengjiCard::small_joker(1)];
    hand.extend((0..jokers - 2).map(|index| ShengjiCard::big_joker(index as u8)));
    for (suit, count) in [
        (ShengjiSuit::Spade, spades),
        (ShengjiSuit::Diamond, 25 - jokers - spades),
    ] {
        hand.extend(
            build_deck()
                .into_iter()
                .filter(|card| card.suit() == Some(suit) && card.rank() != ShengjiRank::Two)
                .take(count),
        );
    }
    let mut deck = build_deck();
    for (index, card) in hand.into_iter().enumerate() {
        let source = deck
            .iter()
            .position(|candidate| *candidate == card)
            .unwrap();
        deck.swap(source, index * 4 + usize::from(player.0));
    }
    let bidder = if player.0 == 1 { 0 } else { 1 };
    let level = ShengjiCard::suited(0, ShengjiSuit::Spade, ShengjiRank::Two);
    let source = deck.iter().position(|card| *card == level).unwrap();
    deck.swap(source, bidder);
    let mut game = ShengjiGameState::new(
        ShengjiRuleSet {
            bottom_copy,
            ..ShengjiRuleSet::default()
        },
        ShengjiTeamProgress::default(),
        Some(PlayerId(0)),
        PlayerId(0),
        deck,
    )
    .unwrap();
    game.deal_all().unwrap();
    game
}

#[test]
fn no_trump_opening_requires_one_third_of_the_normal_trump_share() {
    for (bottom_copy, jokers, expected) in [
        (false, 2, false),
        (false, 3, true),
        (true, 3, false),
        (true, 4, true),
    ] {
        let player = PlayerId(1);
        let game = game_with_hand(player, jokers, 0, bottom_copy);
        let observation = ShengjiBiddingObservation::from_game(&game, player).unwrap();
        assert_eq!(ShengjiBot::declaration(&observation).is_some(), expected);
    }
}

#[test]
fn no_trump_counters_apply_the_reduced_threshold_for_each_role() {
    for (player, spades, expected) in [
        (0, 15, true),
        (0, 16, false),
        (2, 17, true),
        (2, 18, false),
        (1, 13, true),
        (1, 14, false),
    ] {
        let player = PlayerId(player);
        let mut game = game_with_hand(player, 4, spades, false);
        let bidder = PlayerId(if player.0 == 1 { 0 } else { 1 });
        game.declare(
            bidder,
            &[ShengjiCard::suited(0, ShengjiSuit::Spade, ShengjiRank::Two)],
        )
        .unwrap();
        let observation = ShengjiBiddingObservation::from_game(&game, player).unwrap();
        assert_eq!(
            ShengjiBot::declaration(&observation).is_some(),
            expected,
            "{player:?}/{spades}"
        );
    }
}
