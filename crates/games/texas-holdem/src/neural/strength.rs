use crate::{TexasCard, TexasHandCategory, TexasRuleSet, TexasSuit, evaluate_best};
use leocard_ai::TensorMap;

pub(super) fn preflop(hand: &[TexasCard], chart: &TensorMap) -> f32 {
    let mut ranks = [hand[0].rank().value(), hand[1].rank().value()];
    ranks.sort_unstable_by(|a, b| b.cmp(a));
    let rank = |rank: u8| b"23456789TJQKA"[(rank - 2) as usize] as char;
    let mut key = format!("{}{}", rank(ranks[0]), rank(ranks[1]));
    if ranks[0] != ranks[1] {
        key.push(if hand[0].suit() == hand[1].suit() {
            's'
        } else {
            'o'
        });
    }
    lookup(chart, &format!("/preflop/6/values/{key}/strength")).unwrap_or(0.0)
}

pub(super) fn postflop(hand: &[TexasCard], board: &[TexasCard], chart: &TensorMap) -> f32 {
    let cards: Vec<_> = hand.iter().chain(board).copied().collect();
    let best = evaluate_best(&cards, &TexasRuleSet::default()).expect("public showdown");
    let (category, components) = match best.category() {
        TexasHandCategory::HighCard => (0, 5),
        TexasHandCategory::OnePair => (1, 4),
        TexasHandCategory::TwoPair => (2, 3),
        TexasHandCategory::ThreeOfAKind => (3, 3),
        TexasHandCategory::Straight => (4, 1),
        TexasHandCategory::Flush => (5, 5),
        TexasHandCategory::FullHouse => (6, 2),
        TexasHandCategory::FourOfAKind => (7, 2),
        TexasHandCategory::StraightFlush | TexasHandCategory::RoyalFlush => (8, 1),
    };
    let flush_draw = TexasSuit::ALL
        .iter()
        .any(|suit| cards.iter().filter(|c| c.suit() == *suit).count() == 4);
    let mut ranks: Vec<_> = cards.iter().map(|card| card.rank().value()).collect();
    if ranks.contains(&14) {
        ranks.push(1);
    }
    ranks.sort_unstable();
    ranks.dedup();
    let mut maximum = 1;
    let mut run = 1;
    for pair in ranks.windows(2) {
        run = if pair[0] + 1 == pair[1] { run + 1 } else { 1 };
        maximum = maximum.max(run);
    }
    let kickers = best.kickers();
    let mut numerator = 0.0;
    let mut denominator = 0.0;
    for (index, kicker) in kickers.iter().take(components).enumerate() {
        let weight = 15.0_f64.powi(index as i32 + 1);
        numerator += f64::from(*kicker) / weight;
        denominator += 14.0 / weight;
    }
    let bin = ((numerator / denominator * 4.0) as u8).min(3);
    let fd = u8::from(category < 5 && flush_draw);
    let sd = u8::from(category < 4 && maximum >= 4);
    let usage = best
        .cards()
        .iter()
        .filter(|card| hand.contains(card))
        .count();
    let paired = u8::from(
        board
            .iter()
            .enumerate()
            .any(|(i, card)| board[..i].iter().any(|other| other.rank() == card.rank())),
    );
    for key in [
        format!("river|c{category}|b{bin}|fd{fd}|sd{sd}|hu{usage}|bp{paired}"),
        format!("river|c{category}|b{bin}|fd{fd}|sd{sd}"),
        format!("river|c{category}|fd{fd}|sd{sd}"),
        format!("river|c{category}|b{bin}"),
        format!("river|c{category}"),
    ] {
        let key = format!("/postflop/6/river/values/{key}");
        if lookup(chart, &format!("{key}/samples")).is_some_and(|n| n >= 12.0)
            && let Some(value) = lookup(chart, &format!("{key}/strength"))
        {
            return value;
        }
    }
    (category as f32 / 8.0
        + if flush_draw { 0.08 } else { 0.0 }
        + if maximum == 4 { 0.06 } else { 0.0 })
    .min(1.0)
}

fn lookup(chart: &TensorMap, key: &str) -> Option<f32> {
    chart.get(key)?.values.first().copied()
}
