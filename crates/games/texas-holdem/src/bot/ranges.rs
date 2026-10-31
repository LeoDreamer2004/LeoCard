use super::TexasBotObservation;
use super::observation::Opponent;
use crate::{
    TexasAction, TexasCard, TexasHandCategory, TexasRuleSet, TexasStreet, evaluate_player_hand,
};

pub(super) fn starting_strength(cards: &[TexasCard], rules: &TexasRuleSet) -> f64 {
    let mut pairs = Vec::new();
    for left in 0..cards.len() {
        for right in left + 1..cards.len() {
            let a = cards[left].rank().value().max(cards[right].rank().value());
            let b = cards[left].rank().value().min(cards[right].rank().value());
            let high = match a {
                14 => 10.0,
                13 => 8.0,
                12 => 7.0,
                11 => 6.0,
                _ => f64::from(a) / 2.0,
            };
            let gap = a - b;
            let mut strength = if gap == 0 {
                (high * 2.0).max(5.0)
            } else {
                high + if cards[left].suit() == cards[right].suit() {
                    2.0
                } else {
                    0.0
                } - match gap {
                    1 => 0.0,
                    2 => 1.0,
                    3 => 2.0,
                    4 => 4.0,
                    _ => 5.0,
                } + if gap <= 2 && a < 12 { 1.0 } else { 0.0 }
            };
            if rules.short_deck && gap <= 2 {
                strength += 1.5;
            }
            pairs.push((strength / 20.0).clamp(0.0, 1.0));
        }
    }
    pairs.sort_by(|left, right| right.total_cmp(left));
    if !rules.omaha {
        return pairs[0];
    }
    let suits: Vec<_> = cards.iter().map(|card| card.suit()).collect();
    let double_suited = suits
        .iter()
        .filter(|suit| suits.iter().filter(|other| *other == *suit).count() == 2)
        .count()
        == 4;
    let duplicate_ranks = cards
        .iter()
        .filter(|card| {
            cards
                .iter()
                .filter(|other| other.rank() == card.rank())
                .count()
                >= 3
        })
        .count();
    (pairs[0] * 0.5
        + pairs.iter().sum::<f64>() / pairs.len() as f64 * 0.5
        + if double_suited { 0.12 } else { 0.0 }
        - duplicate_ranks as f64 * 0.06)
        .clamp(0.0, 1.0)
}

pub(super) fn range_weight(
    cards: &[TexasCard],
    observation: &TexasBotObservation,
    opponent: &Opponent,
) -> f64 {
    let mut threshold: f64 = 0.08;
    let mut aggression: f64 = 0.0;
    for record in observation
        .history
        .iter()
        .filter(|record| record.player == opponent.id)
    {
        match record.action {
            TexasAction::RaiseTo(_) | TexasAction::AllIn => {
                let pressure = f64::from(record.payment) / f64::from(record.pot_before.max(2));
                if record.street == TexasStreet::PreFlop {
                    threshold = threshold.max(0.28 + (pressure * 0.1).min(0.22));
                } else {
                    aggression = aggression.max((pressure * 0.12).min(0.3));
                }
            }
            TexasAction::Call if record.street == TexasStreet::PreFlop => {
                threshold = threshold.max(0.18)
            }
            _ => {}
        }
    }
    let preflop = starting_strength(cards, &observation.rules);
    let mut weight = (0.08 + 0.92 / (1.0 + ((threshold - preflop) * 15.0).exp())).clamp(0.05, 1.0);
    if aggression > 0.0
        && observation.board.len() >= 3
        && let Ok(hand) = evaluate_player_hand(cards, &observation.board, &observation.rules)
    {
        let made = match hand.category() {
            TexasHandCategory::HighCard => 0.05,
            TexasHandCategory::OnePair => 0.35,
            TexasHandCategory::TwoPair => 0.65,
            _ => 0.9,
        };
        weight *= (0.1 + 0.9 / (1.0 + ((aggression + 0.15 - made) * 12.0).exp())).clamp(0.08, 1.0);
    }
    weight
}
