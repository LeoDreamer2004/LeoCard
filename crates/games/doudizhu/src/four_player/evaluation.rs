use crate::{DoudizhuCard, DoudizhuRuleSet};
use std::collections::HashMap;

pub(super) struct HandEvaluator {
    rules: DoudizhuRuleSet,
    cache: HashMap<[u8; 15], f64>,
}

impl HandEvaluator {
    pub(super) fn new(rules: DoudizhuRuleSet) -> Self {
        Self {
            rules,
            cache: HashMap::new(),
        }
    }

    /// Approximate remaining leads under several competing decompositions.
    /// Runs compete for ranks; bombs, pair wings and high-card control are retained.
    pub(super) fn cost(&mut self, hand: &[DoudizhuCard]) -> f64 {
        let counts = rank_counts(hand);
        if let Some(cost) = self.cache.get(&counts) {
            return *cost;
        }
        let mut best = f64::INFINITY;
        for order in [[3, 2, 1], [3, 1, 2], [2, 1, 3]] {
            let mut remaining = counts;
            let mut turns = 0;
            if remaining[13] == 2 && remaining[14] == 2 {
                remaining[13] = 0;
                remaining[14] = 0;
                turns += 1;
            }
            for count in &mut remaining[..13] {
                if *count >= 4 {
                    *count = 0;
                    turns += 1;
                }
            }
            for width in order {
                let minimum = match width {
                    1 => 5,
                    2 => 3,
                    _ => 2,
                };
                while let Some((start, end)) = longest_run(&remaining, width, minimum) {
                    for count in &mut remaining[start..end] {
                        *count -= width;
                    }
                    turns += 1;
                    if width == 3 {
                        take_wings(
                            &mut remaining,
                            end - start,
                            self.rules.allows_single_wings(),
                        );
                    }
                }
            }
            // Remaining triples can carry pairs (or singles when the variant allows it).
            for rank in 0..13 {
                if remaining[rank] == 3 {
                    remaining[rank] = 0;
                    turns += 1;
                    take_wings(&mut remaining, 1, self.rules.allows_single_wings());
                }
            }
            turns += remaining
                .iter()
                .map(|count| usize::from(count.div_ceil(2)))
                .sum::<usize>();
            let singles = remaining.iter().filter(|count| **count == 1).count();
            best = best.min(turns as f64 + singles as f64 * 0.14);
        }
        let control = counts
            .iter()
            .enumerate()
            .map(|(rank, count)| {
                if rank >= 11 {
                    f64::from(*count) * (rank - 10) as f64 * 0.025
                } else {
                    0.0
                }
            })
            .sum::<f64>();
        let cost = best - control;
        if self.cache.len() >= 8192 {
            self.cache.clear();
        }
        self.cache.insert(counts, cost);
        cost
    }
}

pub(super) fn rank_counts(hand: &[DoudizhuCard]) -> [u8; 15] {
    let mut counts = [0; 15];
    for card in hand {
        counts[usize::from(card.rank())] += 1;
    }
    counts
}

fn longest_run(counts: &[u8; 15], width: u8, minimum: usize) -> Option<(usize, usize)> {
    let mut best = None;
    let mut start = 0;
    while start < 12 {
        if counts[start] < width {
            start += 1;
            continue;
        }
        let mut end = start + 1;
        while end < 12 && counts[end] >= width {
            end += 1;
        }
        if end - start >= minimum && best.is_none_or(|(left, right)| end - start > right - left) {
            best = Some((start, end));
        }
        start = end;
    }
    best
}

fn take_wings(counts: &mut [u8; 15], needed: usize, singles_allowed: bool) {
    let pairs: Vec<_> = (0..15)
        .filter(|rank| counts[*rank] == 2)
        .take(needed)
        .collect();
    if pairs.len() == needed {
        for rank in pairs {
            counts[rank] = 0;
        }
    } else if singles_allowed {
        let singles: Vec<_> = (0..15)
            .filter(|rank| counts[*rank] == 1)
            .take(needed)
            .collect();
        if singles.len() == needed {
            for rank in singles {
                counts[rank] = 0;
            }
        }
    }
}
