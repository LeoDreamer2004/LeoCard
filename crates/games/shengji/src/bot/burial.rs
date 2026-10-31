use super::hand::{counts, face, hand_value};
use crate::{ShengjiCard, ShengjiRuleSet, ShengjiTrump, build_deck_for};
use leocard_game_common::SearchBudget;

pub(super) fn bury(
    hand: &[ShengjiCard],
    count: usize,
    trump: ShengjiTrump,
    rules: ShengjiRuleSet,
    budget: &SearchBudget,
) -> Vec<ShengjiCard> {
    let mut sorted = hand.to_vec();
    sorted.sort_by_key(|card| (trump.is_trump(*card), trump.strength(*card), card.points()));
    let mut best = sorted[..count].to_vec();
    let mut value = evaluate(hand, &best, trump, rules);
    let mut starts = vec![best.clone()];
    for mask in 1_u8..16 {
        let mut cards: Vec<_> = sorted
            .iter()
            .copied()
            .filter(|card| {
                !trump.is_trump(*card)
                    && card
                        .suit()
                        .is_some_and(|suit| mask & (1 << suit as u8) != 0)
            })
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
    for selected in &starts {
        let candidate = evaluate(hand, selected, trump, rules);
        if candidate > value {
            value = candidate;
            best = selected.clone();
        }
    }
    for mut selected in starts {
        for _ in 0..4 {
            if budget.exhausted() {
                return best;
            }
            let mut next = None;
            let mut current_value = evaluate(hand, &selected, trump, rules);
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
                    let new_value = evaluate(hand, &selected, trump, rules);
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
        let current = evaluate(hand, &selected, trump, rules);
        if current > value {
            value = current;
            best = selected;
        }
    }
    best
}

fn evaluate(
    hand: &[ShengjiCard],
    buried: &[ShengjiCard],
    trump: ShengjiTrump,
    rules: ShengjiRuleSet,
) -> f64 {
    let kept: Vec<_> = hand
        .iter()
        .copied()
        .filter(|card| !buried.contains(card))
        .collect();
    let trumps: Vec<_> = kept
        .iter()
        .copied()
        .filter(|card| trump.is_trump(*card))
        .collect();
    let total = build_deck_for(rules.deck_count)
        .into_iter()
        .filter(|card| trump.is_trump(*card))
        .count();
    let average = (total - trumps.len()) as f64 / f64::from(rules.player_count() - 1);
    let high = trumps
        .iter()
        .filter(|card| card.suit().is_none() || card.rank() == trump.level)
        .count();
    let copies = counts(&trumps);
    let pairs = trumps
        .iter()
        .filter(|card| copies[face(**card)] >= 2)
        .count();
    let control = ((trumps.len() as f64 / average.max(1.0) - 0.7) * 0.5
        + high as f64 * 0.07
        + pairs as f64 * 0.04)
        .clamp(0.0, 1.0);
    let points = buried
        .iter()
        .map(|card| f64::from(card.points()))
        .sum::<f64>();
    let master_loss = buried
        .iter()
        .filter(|card| !trump.is_trump(**card) && trump.strength(**card) >= 10)
        .count() as f64
        * 2.0;
    hand_value(&kept, trump) - points * (0.9 - control * 0.65) - master_loss
}
