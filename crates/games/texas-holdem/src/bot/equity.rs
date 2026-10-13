use super::TexasHoldemBotObservation;
use super::ranges::range_weight;
use crate::{TexasHoldemCard, build_deck, evaluate_player_hand};
use leocard_game_common::{SearchBudget, SearchRandom};
use std::cmp::Ordering;

pub(super) fn equity(observation: &TexasHoldemBotObservation, budget: &SearchBudget) -> f64 {
    let unseen: Vec<_> = build_deck(observation.rules.short_deck)
        .into_iter()
        .filter(|card| !observation.hole.contains(card) && !observation.board.contains(card))
        .collect();
    if observation.board.len() == 5 && observation.opponents.len() == 1 && !observation.rules.omaha
    {
        return river_equity(observation, &unseen, budget);
    }
    let mut random = SearchRandom::new(observation.seed());
    let mut samples = 0;
    let mut shares = 0.0;
    let mut squares = 0.0;
    while samples < 8192 && !budget.exhausted() {
        let mut deck = unseen.clone();
        let mut hands = Vec::new();
        for opponent in &observation.opponents {
            let mut selected = Vec::new();
            for _ in 0..48 {
                selected.clear();
                while selected.len() < observation.rules.hole_card_count() {
                    let card = deck[random.index(deck.len())];
                    if !selected.contains(&card) {
                        selected.push(card);
                    }
                }
                if random.unit() < range_weight(&selected, observation, opponent) {
                    break;
                }
            }
            deck.retain(|card| !selected.contains(card));
            hands.push(selected);
        }
        let mut board = observation.board.clone();
        while board.len() < 5 {
            let index = random.index(deck.len());
            board.push(deck.swap_remove(index));
        }
        let ours = evaluate_player_hand(&observation.hole, &board, &observation.rules)
            .expect("valid bot cards");
        let mut winners = 1;
        let mut share = 1.0;
        for hand in &hands {
            let other = evaluate_player_hand(hand, &board, &observation.rules)
                .expect("valid simulated cards");
            match ours.cmp_with_rules(&other, &observation.rules) {
                Ordering::Less => {
                    share = 0.0;
                    break;
                }
                Ordering::Equal => winners += 1,
                Ordering::Greater => {}
            }
        }
        share /= f64::from(winners);
        samples += 1;
        shares += share;
        squares += share * share;
        if samples >= 2048 && samples % 256 == 0 {
            let mean = shares / f64::from(samples);
            let error =
                ((squares / f64::from(samples) - mean * mean).max(0.0) / f64::from(samples)).sqrt();
            if error < 0.008 {
                break;
            }
        }
    }
    if samples == 0 {
        return 0.0;
    }
    shares / f64::from(samples)
}

fn river_equity(
    observation: &TexasHoldemBotObservation,
    unseen: &[TexasHoldemCard],
    budget: &SearchBudget,
) -> f64 {
    let ours = evaluate_player_hand(&observation.hole, &observation.board, &observation.rules)
        .expect("river hand");
    let mut shares = 0.0;
    let mut mass = 0.0;
    let mut random = SearchRandom::new(observation.seed());
    let mut order = unseen.to_vec();
    for index in (1..order.len()).rev() {
        let other = random.index(index + 1);
        order.swap(index, other);
    }
    for left in 0..order.len() {
        for right in left + 1..order.len() {
            if budget.exhausted() {
                return if mass > 0.0 { shares / mass } else { 0.0 };
            }
            let hand = [order[left], order[right]];
            let weight = range_weight(&hand, observation, &observation.opponents[0]);
            let other = evaluate_player_hand(&hand, &observation.board, &observation.rules)
                .expect("river opponent");
            shares += weight
                * match ours.cmp_with_rules(&other, &observation.rules) {
                    Ordering::Greater => 1.0,
                    Ordering::Equal => 0.5,
                    Ordering::Less => 0.0,
                };
            mass += weight;
        }
    }
    shares / mass
}
