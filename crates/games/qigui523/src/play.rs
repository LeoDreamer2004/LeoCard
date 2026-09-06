#[cfg(test)]
#[path = "play_tests.rs"]
mod tests;

use crate::{QiGuiCard, QiGuiRank, QiGuiRuleSet, SameCardPolicy, SuitComparison};
use BombKind::OfAKind;
use QiGuiPlayKind::{Bomb, HeavenBomb};
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum BombKind {
    OfAKind { card_count: usize, rank: QiGuiRank },
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum QiGuiPlayKind {
    Single,
    Pair,
    Straight {
        card_count: usize,
    },
    ConsecutivePairs {
        pair_count: usize,
    },
    Triple,
    /// 一组三张相同点数的主体，附带一张任意其他点数的牌。
    TripleWithSingle,
    /// 一组三张相同点数的主体，附带一对其他点数的牌。
    TripleWithPair,
    /// 至少两组按牌力连续的三张牌，例如 666777。
    Airplane {
        triple_count: usize,
    },
    Bomb(BombKind),
    /// 恰好由 7、王、5、2、3 各一张组成，能够压过其他所有牌型。
    HeavenBomb,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClassifiedPlay {
    cards: Vec<QiGuiCard>,
    kind: QiGuiPlayKind,
}

impl ClassifiedPlay {
    pub fn cards(&self) -> &[QiGuiCard] {
        &self.cards
    }

    pub fn kind(&self) -> &QiGuiPlayKind {
        &self.kind
    }

    pub fn score(&self) -> u16 {
        self.cards.iter().map(|card| card.score()).sum()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlayComparison {
    Greater,
    Equivalent,
    Lower,
    Incompatible,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PlayError {
    Empty,
    DuplicatePhysicalCard(QiGuiCard),
    CardOutsideConfiguredDeck(QiGuiCard),
    InvalidPattern,
}

impl fmt::Display for PlayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("出牌不能为空"),
            Self::DuplicatePhysicalCard(card) => write!(f, "同一张物理牌被重复提交：{card}"),
            Self::CardOutsideConfiguredDeck(card) => write!(f, "牌不属于当前配置的牌堆：{card}"),
            Self::InvalidPattern => f.write_str("这些牌不能组成合法牌型"),
        }
    }
}

impl std::error::Error for PlayError {}

pub fn classify(cards: &[QiGuiCard], rules: &QiGuiRuleSet) -> Result<ClassifiedPlay, PlayError> {
    if cards.is_empty() {
        return Err(PlayError::Empty);
    }

    validate_physical_cards(cards, rules)?;

    let mut sorted_cards = cards.to_vec();
    sorted_cards.sort_by(QiGuiCard::display_cmp);

    let rank_counts = rank_counts(&sorted_cards);
    let card_count = sorted_cards.len();

    let kind = if is_heaven_bomb(&rank_counts, card_count) {
        QiGuiPlayKind::HeavenBomb
    } else if rank_counts.len() == 1 && card_count >= 4 {
        QiGuiPlayKind::Bomb(BombKind::OfAKind {
            card_count,
            rank: sorted_cards[0].rank(),
        })
    } else if card_count == 1 {
        QiGuiPlayKind::Single
    } else if card_count == 2 && rank_counts.len() == 1 {
        QiGuiPlayKind::Pair
    } else if card_count == 3 && rank_counts.len() == 1 {
        QiGuiPlayKind::Triple
    } else if rules.advanced_play_types
        && card_count == 4
        && rank_counts.len() == 2
        && rank_counts.values().any(|count| *count == 3)
        && rank_counts.values().any(|count| *count == 1)
    {
        QiGuiPlayKind::TripleWithSingle
    } else if rules.advanced_play_types
        && card_count == 5
        && rank_counts.len() == 2
        && rank_counts.values().any(|count| *count == 3)
        && rank_counts.values().any(|count| *count == 2)
    {
        QiGuiPlayKind::TripleWithPair
    } else if card_count >= 6
        && card_count.is_multiple_of(3)
        && rank_counts.values().all(|count| *count == 3)
        && ranks_are_consecutive(rank_counts.keys().copied())
    {
        QiGuiPlayKind::Airplane {
            triple_count: card_count / 3,
        }
    } else if card_count >= 3
        && rank_counts.values().all(|count| *count == 1)
        && ranks_are_consecutive(rank_counts.keys().copied())
    {
        QiGuiPlayKind::Straight { card_count }
    } else if card_count >= 4
        && card_count.is_multiple_of(2)
        && rank_counts.values().all(|count| *count == 2)
        && ranks_are_consecutive(rank_counts.keys().copied())
    {
        QiGuiPlayKind::ConsecutivePairs {
            pair_count: card_count / 2,
        }
    } else {
        return Err(PlayError::InvalidPattern);
    };

    Ok(ClassifiedPlay {
        cards: sorted_cards,
        kind,
    })
}

fn validate_physical_cards(cards: &[QiGuiCard], rules: &QiGuiRuleSet) -> Result<(), PlayError> {
    let mut physical_cards = HashSet::with_capacity(cards.len());
    for &card in cards {
        #[cfg(not(feature = "developer"))]
        if card.deck() >= rules.deck_count {
            return Err(PlayError::CardOutsideConfiguredDeck(card));
        }
        if !physical_cards.insert(card) {
            return Err(PlayError::DuplicatePhysicalCard(card));
        }
    }
    #[cfg(feature = "developer")]
    let _ = rules;
    Ok(())
}

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
        (HeavenBomb, HeavenBomb) => {
            semantic_comparison(challenger.cards(), current.cards(), rules.suit_comparison)
        }
        (HeavenBomb, _) => PlayComparison::Greater,
        (_, HeavenBomb) => PlayComparison::Lower,
        (Bomb(OfAKind { .. }), kind) if !matches!(kind, Bomb(_)) => PlayComparison::Greater,
        (kind, Bomb(OfAKind { .. })) if !matches!(kind, Bomb(_)) => PlayComparison::Lower,
        (
            Bomb(OfAKind {
                card_count: challenger_count,
                ..
            }),
            Bomb(OfAKind {
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

fn rank_counts(cards: &[QiGuiCard]) -> HashMap<QiGuiRank, usize> {
    let mut result = HashMap::new();
    for card in cards {
        *result.entry(card.rank()).or_insert(0) += 1;
    }
    result
}

fn is_heaven_bomb(rank_counts: &HashMap<QiGuiRank, usize>, card_count: usize) -> bool {
    const REQUIRED_RANKS: [QiGuiRank; 5] = [
        QiGuiRank::Seven,
        QiGuiRank::Joker,
        QiGuiRank::Five,
        QiGuiRank::Two,
        QiGuiRank::Three,
    ];
    card_count == REQUIRED_RANKS.len()
        && rank_counts.len() == REQUIRED_RANKS.len()
        && rank_counts.values().all(|count| *count == 1)
        && REQUIRED_RANKS
            .into_iter()
            .all(|rank| rank_counts.contains_key(&rank))
}

fn ranks_are_consecutive(ranks: impl IntoIterator<Item = QiGuiRank>) -> bool {
    let mut strengths: Vec<_> = ranks.into_iter().map(QiGuiRank::strength).collect();
    strengths.sort_unstable();
    strengths
        .windows(2)
        .all(|window| window[1] == window[0] + 1)
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
