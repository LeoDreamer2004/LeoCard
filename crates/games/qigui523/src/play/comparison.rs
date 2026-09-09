use super::{BombKind, ClassifiedPlay, PlayComparison, QiGuiPlayKind, rank_counts};
use crate::{QiGuiCard, QiGuiRuleSet, SameCardPolicy, SuitComparison};
use std::cmp::Ordering;

pub fn compare_plays(
    challenger: &ClassifiedPlay,
    current: &ClassifiedPlay,
    rules: &QiGuiRuleSet,
) -> PlayComparison {
    if rules.advanced_play_types {
        let comparison = advanced_type_comparison(challenger.kind(), current.kind());
        if comparison != PlayComparison::Incompatible {
            return comparison;
        }
    }

    match (challenger.kind(), current.kind()) {
        (QiGuiPlayKind::HeavenBomb, QiGuiPlayKind::HeavenBomb) => {
            semantic_comparison(challenger.cards(), current.cards(), rules.suit_comparison)
        }
        (QiGuiPlayKind::HeavenBomb, _) => PlayComparison::Greater,
        (_, QiGuiPlayKind::HeavenBomb) => PlayComparison::Lower,
        (QiGuiPlayKind::Bomb(BombKind::OfAKind { .. }), kind)
            if !matches!(kind, QiGuiPlayKind::Bomb(_)) =>
        {
            PlayComparison::Greater
        }
        (kind, QiGuiPlayKind::Bomb(BombKind::OfAKind { .. }))
            if !matches!(kind, QiGuiPlayKind::Bomb(_)) =>
        {
            PlayComparison::Lower
        }
        (
            QiGuiPlayKind::Bomb(BombKind::OfAKind {
                card_count: challenger_count,
                ..
            }),
            QiGuiPlayKind::Bomb(BombKind::OfAKind {
                card_count: current_count,
                ..
            }),
        ) => match challenger_count.cmp(current_count) {
            Ordering::Greater => PlayComparison::Greater,
            Ordering::Less => PlayComparison::Lower,
            Ordering::Equal => {
                semantic_comparison(challenger.cards(), current.cards(), rules.suit_comparison)
            }
        },
        (QiGuiPlayKind::TripleWithSingle, QiGuiPlayKind::TripleWithSingle)
        | (QiGuiPlayKind::TripleWithPair, QiGuiPlayKind::TripleWithPair) => {
            compare_triple_components(challenger.cards(), current.cards(), rules.suit_comparison)
        }
        (challenger_kind, current_kind) if same_non_bomb_shape(challenger_kind, current_kind) => {
            semantic_comparison(challenger.cards(), current.cards(), rules.suit_comparison)
        }
        _ => PlayComparison::Incompatible,
    }
}

pub fn can_beat(
    challenger: &ClassifiedPlay,
    current: &ClassifiedPlay,
    rules: &QiGuiRuleSet,
) -> bool {
    match compare_plays(challenger, current, rules) {
        PlayComparison::Greater => true,
        PlayComparison::Equivalent => rules.same_card_policy == SameCardPolicy::CanFollow,
        PlayComparison::Lower | PlayComparison::Incompatible => false,
    }
}

fn advanced_type_comparison(challenger: &QiGuiPlayKind, current: &QiGuiPlayKind) -> PlayComparison {
    match (challenger, current) {
        (QiGuiPlayKind::Triple, QiGuiPlayKind::Straight { card_count: 3 })
        | (
            QiGuiPlayKind::ConsecutivePairs { pair_count: 2 },
            QiGuiPlayKind::Straight { card_count: 4 },
        )
        | (
            QiGuiPlayKind::Airplane { triple_count: 2 },
            QiGuiPlayKind::ConsecutivePairs { pair_count: 3 },
        ) => PlayComparison::Greater,
        (QiGuiPlayKind::Straight { card_count: 3 }, QiGuiPlayKind::Triple)
        | (
            QiGuiPlayKind::Straight { card_count: 4 },
            QiGuiPlayKind::ConsecutivePairs { pair_count: 2 },
        )
        | (
            QiGuiPlayKind::ConsecutivePairs { pair_count: 3 },
            QiGuiPlayKind::Airplane { triple_count: 2 },
        ) => PlayComparison::Lower,
        _ => PlayComparison::Incompatible,
    }
}

fn compare_triple_components(
    challenger: &[QiGuiCard],
    current: &[QiGuiCard],
    suit_comparison: SuitComparison,
) -> PlayComparison {
    let challenger = triple_component(challenger);
    let current = triple_component(current);
    semantic_comparison(&challenger, &current, suit_comparison)
}

fn triple_component(cards: &[QiGuiCard]) -> Vec<QiGuiCard> {
    let counts = rank_counts(cards);
    let triple_rank = counts
        .into_iter()
        .find_map(|(rank, count)| (count == 3).then_some(rank))
        .expect("triple-carry play always contains exactly one triple component");
    cards
        .iter()
        .copied()
        .filter(|card| card.rank() == triple_rank)
        .collect()
}

fn same_non_bomb_shape(left: &QiGuiPlayKind, right: &QiGuiPlayKind) -> bool {
    match (left, right) {
        (QiGuiPlayKind::Single, QiGuiPlayKind::Single)
        | (QiGuiPlayKind::Pair, QiGuiPlayKind::Pair)
        | (QiGuiPlayKind::Triple, QiGuiPlayKind::Triple) => true,
        (
            QiGuiPlayKind::Straight {
                card_count: left_count,
            },
            QiGuiPlayKind::Straight {
                card_count: right_count,
            },
        ) => left_count == right_count,
        (
            QiGuiPlayKind::ConsecutivePairs {
                pair_count: left_count,
            },
            QiGuiPlayKind::ConsecutivePairs {
                pair_count: right_count,
            },
        ) => left_count == right_count,
        (
            QiGuiPlayKind::Airplane {
                triple_count: left_count,
            },
            QiGuiPlayKind::Airplane {
                triple_count: right_count,
            },
        ) => left_count == right_count,
        _ => false,
    }
}

fn semantic_comparison(
    left: &[QiGuiCard],
    right: &[QiGuiCard],
    suit_comparison: SuitComparison,
) -> PlayComparison {
    let mut left_ranks: Vec<_> = left.iter().map(|card| card.rank().strength()).collect();
    let mut right_ranks: Vec<_> = right.iter().map(|card| card.rank().strength()).collect();
    left_ranks.sort_unstable_by(|a, b| b.cmp(a));
    right_ranks.sort_unstable_by(|a, b| b.cmp(a));

    let ordering = left_ranks
        .cmp(&right_ranks)
        .then_with(|| match suit_comparison {
            SuitComparison::HighestCard => left
                .iter()
                .map(|card| card.semantic_strength())
                .max()
                .cmp(&right.iter().map(|card| card.semantic_strength()).max()),
            SuitComparison::Lexicographic => {
                let mut left_strengths: Vec<_> =
                    left.iter().map(|card| card.semantic_strength()).collect();
                let mut right_strengths: Vec<_> =
                    right.iter().map(|card| card.semantic_strength()).collect();
                left_strengths.sort_unstable_by(|a, b| b.cmp(a));
                right_strengths.sort_unstable_by(|a, b| b.cmp(a));
                left_strengths.cmp(&right_strengths)
            }
            SuitComparison::SumPoints => left
                .iter()
                .map(|card| card.suit_points())
                .sum::<u16>()
                .cmp(&right.iter().map(|card| card.suit_points()).sum::<u16>()),
        });
    match ordering {
        Ordering::Greater => PlayComparison::Greater,
        Ordering::Equal => PlayComparison::Equivalent,
        Ordering::Less => PlayComparison::Lower,
    }
}
