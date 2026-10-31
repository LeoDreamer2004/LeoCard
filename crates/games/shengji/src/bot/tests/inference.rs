use super::super::endgame;
use super::super::knowledge::{Knowledge, PublicKnowledge};
use super::super::sampling::{DealSampler, SampledDeal};
use super::super::simulation::Simulation;
use super::fixtures::{Table, card, game, play, trump};
use crate::{
    ShengjiBot, ShengjiBotAction, ShengjiBotObservation, ShengjiCard, ShengjiGameState,
    ShengjiMode, ShengjiRank, ShengjiRuleSet, ShengjiSuit, ShengjiTeamId, ShengjiTeamProgress,
    build_deck_for, validate_follow,
};
use leocard_game_common::{PlayerId, SearchBudget, SearchRandom};
use std::collections::HashSet;
use std::time::Duration;

#[test]
fn exhausted_higher_copies_promote_the_next_card_to_master() {
    let king = card(ShengjiSuit::Spade, ShengjiRank::King);
    let ace = card(ShengjiSuit::Spade, ShengjiRank::Ace);
    let table = Table::new(vec![
        vec![king],
        vec![ace],
        vec![ShengjiCard::suited(1, ShengjiSuit::Spade, ShengjiRank::Ace)],
        vec![],
    ]);
    let lengths = [1, 1, 1, 0];
    let mut public = table.public;
    let master = |public: &PublicKnowledge| {
        Knowledge::new(PlayerId(0), &[king], trump(), public, &lengths, None).master(&play(&[king]))
    };
    assert!(!master(&public));
    public.record(PlayerId(1), &play(&[ace]), &play(&[ace]), trump());
    assert!(!master(&public));
    let other = ShengjiCard::suited(1, ShengjiSuit::Spade, ShengjiRank::Ace);
    public.record(PlayerId(2), &play(&[other]), &play(&[other]), trump());
    assert!(master(&public));
}

#[test]
fn failing_to_follow_a_pair_rules_out_remaining_pairs_in_that_suit() {
    let kings = [
        card(ShengjiSuit::Spade, ShengjiRank::King),
        ShengjiCard::suited(1, ShengjiSuit::Spade, ShengjiRank::King),
    ];
    let lower = [
        card(ShengjiSuit::Spade, ShengjiRank::Queen),
        ShengjiCard::suited(1, ShengjiSuit::Spade, ShengjiRank::Queen),
    ];
    let mut table = Table::new(vec![lower.to_vec(), kings.to_vec(), vec![], vec![]]);
    let lead = play(&[
        card(ShengjiSuit::Spade, ShengjiRank::Ace),
        ShengjiCard::suited(1, ShengjiSuit::Spade, ShengjiRank::Ace),
    ]);
    let shown = play(&[
        card(ShengjiSuit::Spade, ShengjiRank::Three),
        card(ShengjiSuit::Spade, ShengjiRank::Four),
    ]);
    table.public.record(PlayerId(1), &shown, &lead, trump());
    let lengths = [2, 2, 0, 0];
    let knowledge = Knowledge::new(PlayerId(0), &lower, trump(), &table.public, &lengths, None);
    assert_eq!(
        knowledge.threat(PlayerId(1), &play(&lower), &play(&lower)),
        0.0
    );
}

#[test]
fn samples_preserve_counts_exposures_and_every_historical_follow_obligation() {
    for mode in [ShengjiMode::Classic, ShengjiMode::SixPlayer] {
        let mut game = game(mode, 708);
        for _ in 0..18 {
            let player = game.current_player().unwrap();
            let ShengjiBotAction::Play(cards) = ShengjiBot::choose(
                &ShengjiBotObservation::from_game(&game, player).unwrap(),
                &SearchBudget::new(Duration::ZERO),
            ) else {
                panic!("play")
            };
            game.play_cards(player, &cards).unwrap();
        }
        let player = game.current_player().unwrap();
        let observation = ShengjiBotObservation::from_game(&game, player).unwrap();
        let public = PublicKnowledge::from_observation(&observation);
        let sampler = DealSampler::new(&observation, &public);
        let deal = sampler
            .sample(
                &mut SearchRandom::new(42),
                &SearchBudget::new(Duration::from_secs(1)),
            )
            .expect("a consistent sample");
        assert_eq!(deal.hands[usize::from(player.0)], observation.hand);
        assert_eq!(
            deal.hands.iter().map(Vec::len).collect::<Vec<_>>(),
            observation.lengths
        );
        assert_eq!(deal.bottom.len(), observation.rules.kitty_size());
        let remaining: Vec<_> = deal
            .hands
            .iter()
            .flatten()
            .chain(&deal.bottom)
            .copied()
            .collect();
        assert_eq!(
            remaining.len(),
            remaining.iter().collect::<HashSet<_>>().len()
        );
        for (hand, exposed) in deal.hands.iter().zip(&observation.exposed) {
            assert!(exposed.iter().all(|card| hand.contains(card)));
        }
        let mut before = deal.hands;
        for trick in observation
            .history
            .iter()
            .chain(observation.trick.iter())
            .rev()
        {
            for (seat, play) in &trick.plays {
                before[usize::from(seat.0)].extend_from_slice(&play.cards);
            }
            for (seat, play) in trick.plays.iter().skip(1) {
                assert!(
                    validate_follow(
                        &before[usize::from(seat.0)],
                        &play.cards,
                        &trick.plays[0].1,
                        observation.trump
                    )
                    .is_ok()
                );
            }
        }
    }
}

#[test]
fn exchanging_hidden_hands_does_not_change_the_observation_or_rule_decision() {
    fn prepare(deck: Vec<ShengjiCard>, rules: ShengjiRuleSet) -> ShengjiGameState {
        let mut game = ShengjiGameState::new(
            rules,
            ShengjiTeamProgress::for_rules(&rules),
            Some(PlayerId(0)),
            PlayerId(0),
            deck,
        )
        .unwrap();
        game.deal_all().unwrap();
        let level = game.players()[0]
            .hand
            .iter()
            .find(|card| card.rank() == ShengjiRank::Two)
            .copied()
            .unwrap();
        game.declare(PlayerId(0), &[level]).unwrap();
        game.close_bidding_and_take_kitty().unwrap();
        let cards = game.players()[0].hand[..rules.kitty_size()].to_vec();
        game.bury(PlayerId(0), &cards).unwrap();
        game
    }
    let rules = ShengjiRuleSet::for_mode(ShengjiMode::SixPlayer);
    let mut deck = build_deck_for(rules.deck_count);
    let first = prepare(deck.clone(), rules);
    let left = first.players()[1]
        .hand
        .iter()
        .find(|card| card.rank() != ShengjiRank::Two)
        .unwrap();
    let right = first.players()[2]
        .hand
        .iter()
        .find(|card| card.rank() != ShengjiRank::Two)
        .unwrap();
    let a = deck.iter().position(|card| card == left).unwrap();
    let b = deck.iter().position(|card| card == right).unwrap();
    deck.swap(a, b);
    let second = prepare(deck, rules);
    assert_ne!(first.players()[1].hand, second.players()[1].hand);
    let observation = ShengjiBotObservation::from_game(&first, PlayerId(0)).unwrap();
    let changed = ShengjiBotObservation::from_game(&second, PlayerId(0)).unwrap();
    assert_eq!(observation, changed);
    assert_eq!(
        ShengjiBot::choose(&observation, &SearchBudget::new(Duration::ZERO)),
        ShengjiBot::choose(&changed, &SearchBudget::new(Duration::ZERO))
    );
}

#[test]
fn exact_six_player_endgame_includes_the_bottom_multiplier_and_takeover_threshold() {
    let hands = vec![
        vec![card(ShengjiSuit::Spade, ShengjiRank::Ace)],
        vec![card(ShengjiSuit::Spade, ShengjiRank::Three)],
        vec![card(ShengjiSuit::Spade, ShengjiRank::King)],
        vec![card(ShengjiSuit::Spade, ShengjiRank::Four)],
        vec![card(ShengjiSuit::Spade, ShengjiRank::Two)],
        vec![card(ShengjiSuit::Heart, ShengjiRank::Three)],
    ];
    let table = Table::new(hands.clone());
    let mut observation =
        ShengjiBotObservation::from_game(&game(ShengjiMode::SixPlayer, 3), PlayerId(0)).unwrap();
    observation.trump = trump();
    observation.trick = None;
    observation.collecting_team = ShengjiTeamId(1);
    observation.collecting_score = 110;
    let position = Simulation::new(
        &observation,
        SampledDeal {
            hands,
            bottom: vec![card(ShengjiSuit::Club, ShengjiRank::Five)],
        },
        &table.public,
    );
    let value = endgame::solve(
        &position,
        PlayerId(0),
        &SearchBudget::new(Duration::from_secs(1)),
        &mut 1000,
    )
    .unwrap();
    assert!((value - (-79.5)).abs() < 0.001);
}
