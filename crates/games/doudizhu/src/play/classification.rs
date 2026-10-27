use super::{Play, PlayKind};
use crate::{Card, RuleSet};
use std::collections::HashSet;

pub fn classify(cards: &[Card], rules: RuleSet) -> Option<Play> {
    let mode = rules.mode;
    if cards.is_empty()
        || cards.iter().any(|c| c.0 >= mode.deck_count() * 54)
        || cards.iter().collect::<HashSet<_>>().len() != cards.len()
    {
        return None;
    }
    let mut counts = [0u8; 15];
    for card in cards {
        counts[usize::from(card.rank())] += 1;
    }
    let ranks: Vec<_> = (0..15).filter(|&r| counts[r] > 0).collect();
    let n = cards.len();
    let mut result = None;
    if counts[13] == mode.deck_count()
        && counts[14] == mode.deck_count()
        && n == usize::from(mode.deck_count()) * 2
    {
        result = Some((PlayKind::Rocket, 14));
    } else if ranks.len() == 1 {
        let rank = ranks[0];
        result = match n {
            1 => Some((PlayKind::Single, rank)),
            2 => Some((PlayKind::Pair, rank)),
            3 if rank < 13 => Some((PlayKind::Triple, rank)),
            4..=8 if rank < 13 => Some((PlayKind::Bomb, rank)),
            _ => None,
        };
    }
    if result.is_none() {
        for (kind, width, minimum) in [
            (PlayKind::Straight, 1, 5),
            (PlayKind::Pairs, 2, 3),
            (PlayKind::Airplane, 3, 2),
        ] {
            if ranks.len() >= minimum
                && counts.iter().all(|&v| v == 0 || v == width)
                && ranks.last().copied()? < 12
                && ranks.windows(2).all(|r| r[1] == r[0] + 1)
            {
                result = Some((kind, *ranks.last()?));
                break;
            }
        }
    }
    if result.is_none() {
        for (kind, wing_width) in [
            (PlayKind::TripleSingle, 1),
            (PlayKind::TriplePair, 2),
            (PlayKind::AirplaneSingles, 1),
            (PlayKind::AirplanePairs, 2),
        ] {
            if wing_width == 1 && !rules.allows_single_wings() {
                continue;
            }
            let group = 3 + wing_width;
            if !n.is_multiple_of(group) {
                continue;
            }
            let length = n / group;
            let single = matches!(kind, PlayKind::TripleSingle | PlayKind::TriplePair);
            if (single && length != 1) || (!single && length < 2) || length > 12 {
                continue;
            }
            let max_start = if single { 13 - length } else { 12 - length };
            for start in 0..=max_start {
                let end = start + length;
                if !counts[start..end].iter().all(|&c| c == 3) {
                    continue;
                }
                let wings: Vec<_> = counts
                    .iter()
                    .enumerate()
                    .filter(|(r, c)| !(start..end).contains(r) && **c > 0)
                    .map(|(_, c)| *c)
                    .collect();
                // A wing cannot reuse a body rank. Pair wings must be distinct pairs.
                let valid = if wing_width == 1 {
                    wings.iter().map(|&v| usize::from(v)).sum::<usize>() == length
                } else {
                    wings.len() == length && wings.iter().all(|&v| v == 2)
                };
                if valid {
                    result = Some((kind, end - 1));
                }
            }
            if result.is_some() {
                break;
            }
        }
    }
    if result.is_none() && matches!(n, 6 | 8) {
        for &rank in &ranks {
            if counts[rank] != 4 || rank >= 13 {
                continue;
            }
            let rest: Vec<_> = ranks.iter().copied().filter(|&r| r != rank).collect();
            if n == 6 {
                result = Some((PlayKind::FourSingles, rank));
            }
            if n == 8 && rest.len() == 2 && rest.iter().all(|&r| counts[r] == 2) {
                result = Some((PlayKind::FourPairs, rank));
            }
        }
    }
    let (kind, rank) = result?;
    let mut cards = cards.to_vec();
    cards.sort_by_key(|c| (c.rank(), c.0));
    Some(Play {
        cards,
        kind,
        rank: rank as u8,
    })
}
