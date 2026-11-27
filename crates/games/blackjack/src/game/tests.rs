use crate::{
    BlackjackAction, BlackjackGameState, BlackjackHandValue, BlackjackObservation, BlackjackRank,
    BlackjackRuleSet, build_shoe,
};
use leocard_game_common::PlayerId;

fn game(ranks: &[BlackjackRank]) -> BlackjackGameState {
    let mut shoe = build_shoe();
    let mut prefix = Vec::new();
    for rank in ranks {
        let index = shoe.iter().position(|card| card.rank == *rank).unwrap();
        prefix.push(shoe.remove(index));
    }
    prefix.extend(shoe);
    BlackjackGameState::new(BlackjackRuleSet::default(), 1, prefix).unwrap()
}

#[test]
fn dealer_hits_soft_seventeen_but_stands_on_hard_seventeen_and_soft_eighteen() {
    for (up, hole, next, count, total) in [
        (
            BlackjackRank::Ace,
            BlackjackRank::Six,
            BlackjackRank::Ten,
            3,
            17,
        ),
        (
            BlackjackRank::Ace,
            BlackjackRank::Six,
            BlackjackRank::Ace,
            3,
            18,
        ),
        (
            BlackjackRank::Ten,
            BlackjackRank::Seven,
            BlackjackRank::Five,
            2,
            17,
        ),
        (
            BlackjackRank::Ace,
            BlackjackRank::Seven,
            BlackjackRank::Five,
            2,
            18,
        ),
    ] {
        let mut game = game(&[BlackjackRank::Ten, up, BlackjackRank::King, hole, next]);
        game.act(PlayerId(0), BlackjackAction::Bet { amount: 2 })
            .unwrap();
        game.act(PlayerId(0), BlackjackAction::Stand).unwrap();
        assert_eq!(game.visible_dealer().len(), count);
        assert_eq!(BlackjackHandValue::of(game.visible_dealer()).total, total);
    }
}

#[test]
fn bot_observation_excludes_dealer_hole_card_and_the_shoe() {
    let prefix = [
        BlackjackRank::Ten,
        BlackjackRank::Six,
        BlackjackRank::King,
        BlackjackRank::Seven,
        BlackjackRank::Five,
    ];
    let mut first = game(&prefix);
    let mut second = game(&prefix);
    let length = second.shoe.len();
    second.shoe.swap(length - 4, length - 5);
    for game in [&mut first, &mut second] {
        game.act(PlayerId(0), BlackjackAction::Bet { amount: 2 })
            .unwrap();
    }
    assert_ne!(first.dealer, second.dealer);
    assert_eq!(
        BlackjackObservation::for_player(&first, PlayerId(0)),
        BlackjackObservation::for_player(&second, PlayerId(0))
    );
}
