use super::candidates::hand_value;
use crate::{ShengjiCard, ShengjiSuit, ShengjiTrump};
use leocard_game_common::SearchBudget;

pub(super) fn bury(
    hand: &[ShengjiCard],
    count: usize,
    trump: ShengjiTrump,
    budget: &SearchBudget,
) -> Vec<ShengjiCard> {
    let mut sorted = hand.to_vec();
    sorted.sort_by_key(|card| (trump.is_trump(*card), trump.strength(*card), card.points()));
    let mut best = sorted[..count].to_vec();
    let mut value = evaluate(hand, &best, trump);
    let mut starts = vec![best.clone()];
    for suit in ShengjiSuit::ALL {
        let mut cards: Vec<_> = sorted
            .iter()
            .copied()
            .filter(|card| !trump.is_trump(*card) && card.suit() == Some(suit))
            .collect();
        if cards.len() > count {
            continue;
        }
        for card in &sorted {
            if cards.len() == count {
                break;
            }
            if !cards.contains(card) {
                cards.push(*card);
            }
        }
        starts.push(cards);
    }
    for mut selected in starts {
        for _ in 0..4 {
            if budget.exhausted() {
                return best;
            }
            let mut next = None;
            let mut current_value = evaluate(hand, &selected, trump);
            for slot in 0..count {
                for &card in hand {
                    if budget.exhausted() {
                        return best;
                    }
                    if selected.contains(&card) {
                        continue;
                    }
                    let old = selected[slot];
                    selected[slot] = card;
                    let new_value = evaluate(hand, &selected, trump);
                    if new_value > current_value + 0.001 {
                        current_value = new_value;
                        next = Some((slot, card));
                    }
                    selected[slot] = old;
                }
            }
            if let Some((slot, card)) = next {
                selected[slot] = card;
            } else {
                break;
            }
            if current_value > value {
                value = current_value;
                best = selected.clone();
            }
        }
        let current = evaluate(hand, &selected, trump);
        if current > value {
            value = current;
            best = selected;
        }
    }
    best
}

fn evaluate(hand: &[ShengjiCard], buried: &[ShengjiCard], trump: ShengjiTrump) -> f64 {
    let kept: Vec<_> = hand
        .iter()
        .copied()
        .filter(|card| !buried.contains(card))
        .collect();
    let control = kept.iter().filter(|card| trump.is_trump(**card)).count() as f64;
    let points = buried
        .iter()
        .map(|card| f64::from(card.points()))
        .sum::<f64>();
    hand_value(&kept, trump) - points * if control >= 12.0 { 0.12 } else { 0.45 }
}
