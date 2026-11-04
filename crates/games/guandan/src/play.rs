use crate::{GuandanCard, GuandanSuit};
use std::{collections::HashSet, error::Error, fmt};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum GuandanPlayKind {
    Single,
    Pair,
    Triple,
    FullHouse,
    Straight,
    ConsecutivePairs,
    ConsecutiveTriples,
    Bomb(u8),
    StraightFlush,
    JokerBomb,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GuandanPlay {
    pub cards: Vec<GuandanCard>,
    pub kind: GuandanPlayKind,
    /// Main rank. Sequences use their natural high card (A2345 is five-high).
    pub value: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GuandanPlayError {
    Empty,
    TooManyCards,
    DuplicateCard,
    InvalidCard,
    InvalidPattern,
}

impl fmt::Display for GuandanPlayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("请选择要出的牌"),
            Self::TooManyCards => f.write_str("单次出牌不能超过十张"),
            Self::DuplicateCard => f.write_str("同一张实体牌不能重复出牌"),
            Self::InvalidCard => f.write_str("牌不属于标准两副牌"),
            Self::InvalidPattern => f.write_str("所选的牌不构成掼蛋牌型"),
        }
    }
}

impl Error for GuandanPlayError {}

pub fn classify(cards: &[GuandanCard], level: u8) -> Result<GuandanPlay, GuandanPlayError> {
    if cards.is_empty() {
        return Err(GuandanPlayError::Empty);
    }
    if cards.len() > 10 {
        return Err(GuandanPlayError::TooManyCards);
    }
    if cards.iter().any(|card| {
        card.deck > 1
            || match card.suit {
                GuandanSuit::Joker => !matches!(card.rank, 16 | 17),
                _ => !(2..=14).contains(&card.rank),
            }
    }) {
        return Err(GuandanPlayError::InvalidCard);
    }
    if cards.iter().copied().collect::<HashSet<_>>().len() != cards.len() {
        return Err(GuandanPlayError::DuplicateCard);
    }
    if !(2..=14).contains(&level) {
        return Err(GuandanPlayError::InvalidPattern);
    }

    let wild = cards.iter().filter(|card| card.is_wild(level)).count();
    let mut counts = [0_u8; 18];
    for card in cards.iter().filter(|card| !card.is_wild(level)) {
        counts[usize::from(card.rank)] += 1;
    }
    let pattern =
        classify_counts(cards, level, wild, &counts).ok_or(GuandanPlayError::InvalidPattern)?;
    Ok(GuandanPlay {
        cards: cards.to_vec(),
        kind: pattern.0,
        value: pattern.1,
    })
}

fn classify_counts(
    cards: &[GuandanCard],
    level: u8,
    wild: usize,
    counts: &[u8; 18],
) -> Option<(GuandanPlayKind, u8)> {
    let len = cards.len();
    if len == 4 && counts[16] == 2 && counts[17] == 2 {
        return Some((GuandanPlayKind::JokerBomb, 17));
    }
    if (4..=10).contains(&len)
        && let Some(rank) = best_rank(counts, wild, &[(len as u8, false)], level)
    {
        return Some((GuandanPlayKind::Bomb(len as u8), rank));
    }
    if len == 5
        && let Some(high) = best_sequence(counts, wild, &[1, 1, 1, 1, 1], level)
        && flush_suit(cards, level)
    {
        return Some((GuandanPlayKind::StraightFlush, high));
    }
    if len == 5 {
        if let Some(rank) = best_full_house(counts, wild, level) {
            return Some((GuandanPlayKind::FullHouse, rank));
        }
        if let Some(high) = best_sequence(counts, wild, &[1, 1, 1, 1, 1], level) {
            return Some((GuandanPlayKind::Straight, high));
        }
    }
    if len == 6 {
        if let Some(high) = best_sequence(counts, wild, &[2, 2, 2], level) {
            return Some((GuandanPlayKind::ConsecutivePairs, high));
        }
        if let Some(high) = best_sequence(counts, wild, &[3, 3], level) {
            return Some((GuandanPlayKind::ConsecutiveTriples, high));
        }
    }
    if (1..=3).contains(&len)
        && let Some(rank) = best_rank(counts, wild, &[(len as u8, true)], level)
    {
        let kind = match len {
            1 => GuandanPlayKind::Single,
            2 => GuandanPlayKind::Pair,
            _ => GuandanPlayKind::Triple,
        };
        return Some((kind, rank));
    }
    None
}

fn best_rank(counts: &[u8; 18], wild: usize, shape: &[(u8, bool)], level: u8) -> Option<u8> {
    let (needed, allow_joker) = shape[0];
    let end = if allow_joker { 17 } else { 14 };
    (2..=end)
        .filter(|&rank| {
            counts[rank] as usize + wild == usize::from(needed)
                && counts
                    .iter()
                    .enumerate()
                    .all(|(other, count)| other == rank || *count == 0)
                && (rank <= 14 || wild == 0)
        })
        .max_by_key(|&rank| rank_strength(rank as u8, level))
        .map(|rank| rank as u8)
}

fn best_full_house(counts: &[u8; 18], wild: usize, level: u8) -> Option<u8> {
    (2..=14)
        .flat_map(|triple| (2..=17).map(move |pair| (triple, pair)))
        .filter(|(triple, pair)| triple != pair && *pair != 15)
        .filter(|(triple, pair)| {
            let needed = (3_usize.saturating_sub(usize::from(counts[*triple])))
                + (2_usize.saturating_sub(usize::from(counts[*pair])));
            needed == wild
                && counts[*triple] <= 3
                && counts[*pair] <= 2
                && (pair <= &14 || counts[*pair] == 2)
                && counts
                    .iter()
                    .enumerate()
                    .all(|(rank, count)| rank == *triple || rank == *pair || *count == 0)
        })
        .max_by_key(|(triple, pair)| {
            (
                rank_strength(*triple as u8, level),
                rank_strength(*pair as u8, level),
            )
        })
        .map(|(triple, _)| triple as u8)
}

fn best_sequence(counts: &[u8; 18], wild: usize, shape: &[u8], _level: u8) -> Option<u8> {
    let size = shape.len();
    let last_start = 15 - size;
    (1..=last_start)
        .filter_map(|start| {
            let ranks: Vec<usize> = (0..size)
                .map(|offset| {
                    let rank = start + offset;
                    if rank == 1 { 14 } else { rank }
                })
                .collect();
            let missing: usize = ranks
                .iter()
                .zip(shape)
                .map(|(rank, target)| usize::from(target.saturating_sub(counts[*rank])))
                .sum();
            let fits = missing == wild
                && ranks
                    .iter()
                    .zip(shape)
                    .all(|(rank, target)| counts[*rank] <= *target)
                && counts
                    .iter()
                    .enumerate()
                    .all(|(rank, count)| *count == 0 || ranks.contains(&rank));
            fits.then_some(if start == 1 {
                size as u8
            } else {
                (start + size - 1) as u8
            })
        })
        .max()
}

fn flush_suit(cards: &[GuandanCard], level: u8) -> bool {
    let mut suit = None;
    for card in cards.iter().filter(|card| !card.is_wild(level)) {
        if card.is_joker() {
            return false;
        }
        if let Some(existing) = suit {
            if card.suit != existing {
                return false;
            }
        } else {
            suit = Some(card.suit);
        }
    }
    suit.is_some()
}

pub fn can_beat(candidate: &GuandanPlay, current: &GuandanPlay, level: u8) -> bool {
    match (bomb_tier(candidate.kind), bomb_tier(current.kind)) {
        (Some(candidate_tier), Some(current_tier)) => {
            candidate_tier > current_tier
                || (candidate_tier == current_tier && compare_value(candidate, current, level))
        }
        (Some(_), None) => true,
        (None, Some(_)) => false,
        (None, None) => candidate.kind == current.kind && compare_value(candidate, current, level),
    }
}

fn compare_value(candidate: &GuandanPlay, current: &GuandanPlay, level: u8) -> bool {
    let sequence = matches!(
        candidate.kind,
        GuandanPlayKind::Straight
            | GuandanPlayKind::ConsecutivePairs
            | GuandanPlayKind::ConsecutiveTriples
            | GuandanPlayKind::StraightFlush
    );
    if sequence {
        candidate.value > current.value
    } else {
        rank_strength(candidate.value, level) > rank_strength(current.value, level)
    }
}

fn bomb_tier(kind: GuandanPlayKind) -> Option<u8> {
    match kind {
        GuandanPlayKind::Bomb(len) if (4..=5).contains(&len) => Some(len),
        GuandanPlayKind::StraightFlush => Some(6),
        GuandanPlayKind::Bomb(len) if (6..=10).contains(&len) => Some(len + 1),
        GuandanPlayKind::JokerBomb => Some(12),
        _ => None,
    }
}

pub(crate) const fn rank_strength(rank: u8, level: u8) -> u8 {
    match rank {
        16 | 17 => rank + 1,
        _ if rank == level => 16,
        _ => rank,
    }
}

#[cfg(test)]
mod tests;
