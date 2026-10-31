use super::super::knowledge::Knowledge;
use super::fixtures::{Table, card, game, play, trump};
use crate::{
    ShengjiBot, ShengjiBotAction, ShengjiBotObservation, ShengjiCard, ShengjiMode, ShengjiPhase,
    ShengjiRank, ShengjiSuit, ShengjiTrickRecord,
};
use leocard_game_common::{PlayerId, SearchBudget};
use std::time::Duration;

#[test]
fn safely_winning_partner_receives_points_without_an_unnecessary_overtake() {
    let five = card(ShengjiSuit::Spade, ShengjiRank::Five);
    let mut table = Table::new(vec![
        vec![card(ShengjiSuit::Club, ShengjiRank::Three)],
        vec![card(ShengjiSuit::Club, ShengjiRank::Four)],
        vec![card(ShengjiSuit::Club, ShengjiRank::Six)],
        vec![five, card(ShengjiSuit::Spade, ShengjiRank::Ace)],
    ]);
    table.trick = Some(ShengjiTrickRecord {
        leader: PlayerId(0),
        winner: PlayerId(1),
        points: 10,
        plays: vec![
            (
                PlayerId(0),
                play(&[card(ShengjiSuit::Spade, ShengjiRank::Two)]),
            ),
            (
                PlayerId(1),
                play(&[card(ShengjiSuit::Spade, ShengjiRank::King)]),
            ),
            (
                PlayerId(2),
                play(&[card(ShengjiSuit::Spade, ShengjiRank::Four)]),
            ),
        ],
    });
    assert_eq!(table.best(3).cards, vec![five]);
}

#[test]
fn three_plays_do_not_make_a_six_player_trick_safe() {
    let low = card(ShengjiSuit::Spade, ShengjiRank::Three);
    let mut table = Table::new(vec![
        vec![card(ShengjiSuit::Spade, ShengjiRank::Six)],
        vec![card(ShengjiSuit::Spade, ShengjiRank::Seven)],
        vec![card(ShengjiSuit::Spade, ShengjiRank::Eight)],
        vec![low, card(ShengjiSuit::Spade, ShengjiRank::Five)],
        vec![
            ShengjiCard::big_joker(0),
            card(ShengjiSuit::Club, ShengjiRank::Three),
        ],
        vec![
            card(ShengjiSuit::Spade, ShengjiRank::Nine),
            card(ShengjiSuit::Spade, ShengjiRank::Jack),
        ],
    ]);
    table.public.voids[4][ShengjiSuit::Spade as usize] = true;
    table.trick = Some(ShengjiTrickRecord {
        leader: PlayerId(0),
        winner: PlayerId(1),
        points: 0,
        plays: vec![
            (
                PlayerId(0),
                play(&[card(ShengjiSuit::Spade, ShengjiRank::Two)]),
            ),
            (
                PlayerId(1),
                play(&[card(ShengjiSuit::Spade, ShengjiRank::Ace)]),
            ),
            (
                PlayerId(2),
                play(&[card(ShengjiSuit::Spade, ShengjiRank::Four)]),
            ),
        ],
    });
    assert_eq!(table.best(3).cards, vec![low]);
}

#[test]
fn both_partners_are_considered_in_six_player_seat_order() {
    let table = Table::new(vec![
        vec![card(ShengjiSuit::Club, ShengjiRank::Three)],
        vec![card(ShengjiSuit::Spade, ShengjiRank::Three)],
        vec![card(ShengjiSuit::Club, ShengjiRank::Four)],
        vec![card(ShengjiSuit::Spade, ShengjiRank::Four)],
        vec![card(ShengjiSuit::Club, ShengjiRank::Five)],
        vec![ShengjiCard::big_joker(0)],
    ]);
    let lengths: Vec<_> = table.hands.iter().map(Vec::len).collect();
    let knowledge = Knowledge::new(
        PlayerId(1),
        &table.hands[1],
        trump(),
        &table.public,
        &lengths,
        None,
    );
    let winner = play(&[card(ShengjiSuit::Heart, ShengjiRank::King)]);
    let alone = knowledge.team_probability(&winner, &winner, PlayerId(0), &[]);
    let with_partner = knowledge.team_probability(&winner, &winner, PlayerId(0), &[PlayerId(5)]);
    assert_eq!(alone, 0.0);
    assert!(with_partner > alone);
}

#[test]
fn drawing_trump_preserves_jokers_and_side_aces_can_still_be_cashed() {
    let mut opening: Vec<_> = [
        ShengjiRank::Three,
        ShengjiRank::Four,
        ShengjiRank::Five,
        ShengjiRank::Six,
        ShengjiRank::Seven,
    ]
    .into_iter()
    .flat_map(|rank| (0..2).map(move |deck| ShengjiCard::suited(deck, ShengjiSuit::Heart, rank)))
    .collect();
    opening.extend([
        ShengjiCard::small_joker(0),
        ShengjiCard::big_joker(0),
        ShengjiCard::big_joker(1),
    ]);
    let mut table = Table::new(vec![
        opening,
        vec![ShengjiCard::small_joker(1)],
        vec![card(ShengjiSuit::Heart, ShengjiRank::King)],
        vec![card(ShengjiSuit::Heart, ShengjiRank::Queen)],
    ]);
    table.rules.allow_throw = false;
    assert!(table.best(0).cards.iter().all(|card| card.suit().is_some()));
    let ace = card(ShengjiSuit::Spade, ShengjiRank::Ace);
    let table = Table::new(vec![
        vec![ace, card(ShengjiSuit::Spade, ShengjiRank::Three)],
        vec![card(ShengjiSuit::Spade, ShengjiRank::King)],
        vec![],
        vec![],
    ]);
    assert_eq!(table.best(0).cards, vec![ace]);
}

#[test]
fn four_and_six_player_bots_finish_legal_hands_with_cancelled_search() {
    for mode in [ShengjiMode::Classic, ShengjiMode::SixPlayer] {
        let mut game = game(mode, 203);
        let budget = SearchBudget::new(Duration::from_secs(1));
        budget.cancel();
        for _ in 0..200 {
            if matches!(game.phase(), ShengjiPhase::Finished(_)) {
                break;
            }
            let player = game.current_player().unwrap();
            let observation = ShengjiBotObservation::from_game(&game, player).unwrap();
            let ShengjiBotAction::Play(cards) = ShengjiBot::choose(&observation, &budget) else {
                panic!("play")
            };
            game.play_cards(player, &cards).unwrap();
        }
        assert!(matches!(game.phase(), ShengjiPhase::Finished(_)));
    }
}
