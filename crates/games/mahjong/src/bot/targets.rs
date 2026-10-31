use super::shanten::{Counts, permutations};
use crate::{MahjongMeld, MahjongMeldKind};

/// A graded preference, not an assumed fan award: terminal positions are scored by the rules.
pub(super) fn potential(counts: &Counts, melds: &[MahjongMeld]) -> f64 {
    let mut all = *counts;
    for meld in melds {
        for tile in meld.tile_kinds() {
            all[tile.index34().expect("meld tile")] += 1;
        }
    }
    let total = all.iter().map(|count| usize::from(*count)).sum::<usize>();
    let closed = melds.iter().all(|meld| !meld.is_open());
    let pairs = counts.iter().map(|count| count / 2).sum::<u8>();
    let mut best = if closed {
        1.5 + f64::from(pairs.saturating_sub(3)) * 1.8
    } else {
        0.0
    };
    for suit in 0..3 {
        let suited = all[suit * 9..suit * 9 + 9]
            .iter()
            .map(|count| usize::from(*count))
            .sum::<usize>();
        let honors = all[27..]
            .iter()
            .map(|count| usize::from(*count))
            .sum::<usize>();
        let full = (suited as f64 - (total - suited) as f64 * 2.0 - 5.0).max(0.0);
        let half =
            (suited as f64 + honors as f64 - (total - suited - honors) as f64 * 2.0 - 6.0).max(0.0);
        best = best.max(full * 0.7).max(half * 0.6);
        let straight = all[suit * 9..suit * 9 + 9]
            .iter()
            .filter(|count| **count > 0)
            .count();
        best = best.max((straight as f64 - 4.0).max(0.0) * 0.7);
    }
    if !melds
        .iter()
        .any(|meld| meld.kind() == MahjongMeldKind::Chow)
    {
        let pungs = counts
            .iter()
            .map(|count| (*count).min(3))
            .filter(|count| *count >= 2)
            .sum::<u8>() as f64
            + melds.len() as f64 * 3.0;
        best = best.max((pungs - 6.0).max(0.0) * 0.65);
    }
    for start in 0..7 {
        let matched = (0..3)
            .flat_map(|suit| (start..start + 3).map(move |rank| suit * 9 + rank))
            .filter(|index| all[*index] > 0)
            .count();
        best = best.max((matched as f64 - 5.0).max(0.0) * 0.85);
    }
    for suits in permutations() {
        for start in 0..5 {
            let matched = suits
                .iter()
                .enumerate()
                .flat_map(|(step, suit)| {
                    (start + step..start + step + 3).map(move |rank| suit * 9 + rank)
                })
                .filter(|index| all[*index] > 0)
                .count();
            best = best.max((matched as f64 - 5.0).max(0.0) * 0.7);
        }
    }
    best
}
