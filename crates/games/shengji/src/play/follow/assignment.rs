use super::super::{ShengjiCard, ShengjiRank, ShengjiSuit, ShengjiTrump, strength};
use super::FollowPattern;
use std::collections::{BTreeMap, HashSet};

#[derive(Clone)]
struct FaceStock {
    cards: Vec<ShengjiCard>,
    strength: u8,
}

fn face_stocks(cards: &[ShengjiCard], trump: ShengjiTrump) -> Vec<FaceStock> {
    let mut faces: BTreeMap<(Option<ShengjiSuit>, ShengjiRank), Vec<ShengjiCard>> = BTreeMap::new();
    for card in cards {
        faces
            .entry((card.suit(), card.rank()))
            .or_default()
            .push(*card);
    }
    let mut stocks = faces
        .into_values()
        .map(|mut cards| {
            cards.sort_by_key(|card| card.deck());
            FaceStock {
                strength: strength(cards[0], trump),
                cards,
            }
        })
        .collect::<Vec<_>>();
    stocks.sort_by_key(|stock| {
        (
            stock.strength,
            stock.cards[0].suit().map_or(4, ShengjiSuit::bid_strength),
            stock.cards[0].rank(),
        )
    });
    stocks
}

struct AssignmentSearch<'a> {
    stocks: &'a [FaceStock],
    pattern: FollowPattern,
    group_widths: &'a [usize],
    exact: bool,
    limit: usize,
    output: Vec<Vec<usize>>,
    complete: bool,
}

impl AssignmentSearch<'_> {
    fn collect(
        &mut self,
        start: Option<u8>,
        offset: usize,
        selected: &mut [usize],
        group_index: usize,
    ) {
        if self.output.len() >= self.limit {
            self.complete = false;
            return;
        }
        if offset < usize::from(self.pattern.run_len) {
            let wanted = start.unwrap() + offset as u8;
            for index in 0..self.stocks.len() {
                let stock = &self.stocks[index];
                if selected[index] == 0
                    && stock.strength == wanted
                    && stock.cards.len() >= usize::from(self.pattern.run_width)
                {
                    selected[index] = usize::from(self.pattern.run_width);
                    self.collect(start, offset + 1, selected, group_index);
                    selected[index] = 0;
                }
            }
            return;
        }
        if group_index < self.group_widths.len() {
            let width = self.group_widths[group_index];
            for index in 0..self.stocks.len() {
                if selected[index] == 0 && self.stocks[index].cards.len() >= width {
                    selected[index] = width;
                    self.collect(start, offset, selected, group_index + 1);
                    selected[index] = 0;
                }
            }
            return;
        }

        let used = selected.iter().sum::<usize>();
        let remaining = self
            .stocks
            .iter()
            .zip(selected.iter())
            .map(|(stock, used)| stock.cards.len() - *used)
            .sum::<usize>();
        let wildcards = usize::from(self.pattern.wildcards);
        if remaining < wildcards || (self.exact && remaining != wildcards) {
            return;
        }
        let mut completed = selected.to_vec();
        let mut needed = wildcards;
        for (index, stock) in self.stocks.iter().enumerate() {
            let available = stock.cards.len() - completed[index];
            let take = available.min(needed);
            completed[index] += take;
            needed -= take;
        }
        debug_assert_eq!(needed, 0);
        debug_assert_eq!(completed.iter().sum::<usize>(), used + wildcards);
        self.output.push(completed);
    }
}

fn pattern_assignments(
    cards: &[ShengjiCard],
    trump: ShengjiTrump,
    pattern: FollowPattern,
    exact: bool,
    limit: usize,
) -> (Vec<Vec<usize>>, bool) {
    let stocks = face_stocks(cards, trump);
    let mut group_widths = Vec::new();
    group_widths.extend(std::iter::repeat_n(4, usize::from(pattern.quads)));
    group_widths.extend(std::iter::repeat_n(3, usize::from(pattern.triples)));
    group_widths.extend(std::iter::repeat_n(2, usize::from(pattern.pairs)));
    group_widths.extend(std::iter::repeat_n(1, usize::from(pattern.singles)));
    let starts = if pattern.run_len == 0 {
        vec![None]
    } else {
        let mut starts = stocks
            .iter()
            .map(|stock| stock.strength)
            .collect::<HashSet<_>>()
            .into_iter()
            .map(Some)
            .collect::<Vec<_>>();
        starts.sort_unstable();
        starts
    };
    let mut search = AssignmentSearch {
        stocks: &stocks,
        pattern,
        group_widths: &group_widths,
        exact,
        limit,
        output: Vec::new(),
        complete: true,
    };
    for start in starts {
        let mut selected = vec![0; stocks.len()];
        search.collect(start, 0, &mut selected, 0);
        if !search.complete {
            break;
        }
    }
    search.output.sort();
    search.output.dedup();
    (search.output, search.complete)
}

pub(crate) fn best_follow_tier(
    cards: &[ShengjiCard],
    trump: ShengjiTrump,
    hierarchy: &[Vec<FollowPattern>],
    exact: bool,
) -> Option<usize> {
    hierarchy.iter().position(|tier| {
        tier.iter().any(|pattern| {
            let (assignments, _) = pattern_assignments(cards, trump, *pattern, exact, 1);
            !assignments.is_empty()
        })
    })
}

pub(super) fn special_follow_card_sets(
    cards: &[ShengjiCard],
    trump: ShengjiTrump,
    hierarchy: &[Vec<FollowPattern>],
    limit: usize,
) -> Vec<Vec<ShengjiCard>> {
    let Some(tier) = best_follow_tier(cards, trump, hierarchy, false) else {
        return Vec::new();
    };
    let stocks = face_stocks(cards, trump);
    let mut output = Vec::new();
    for pattern in &hierarchy[tier] {
        let (assignments, _) = pattern_assignments(cards, trump, *pattern, false, 1);
        for assignment in assignments {
            output.push(cards_for_assignment(&stocks, &assignment, trump));
        }
        if output.len() >= limit {
            break;
        }
    }
    if output.len() < limit {
        for pattern in &hierarchy[tier] {
            let (assignments, _) =
                pattern_assignments(cards, trump, *pattern, false, limit - output.len() + 1);
            for assignment in assignments {
                output.push(cards_for_assignment(&stocks, &assignment, trump));
                if output.len() >= limit {
                    break;
                }
            }
            if output.len() >= limit {
                break;
            }
        }
    }
    let mut seen = HashSet::new();
    output.retain(|candidate| seen.insert(candidate.clone()));
    output.truncate(limit);
    output
}

fn cards_for_assignment(
    stocks: &[FaceStock],
    assignment: &[usize],
    trump: ShengjiTrump,
) -> Vec<ShengjiCard> {
    let mut selected = Vec::new();
    for (stock, count) in stocks.iter().zip(assignment) {
        selected.extend(stock.cards.iter().take(*count).copied());
    }
    selected.sort_by_key(|card| (strength(*card, trump), card.deck()));
    selected
}

pub(super) fn forced_cards_for_hierarchy(
    hand: &[ShengjiCard],
    cards: &[ShengjiCard],
    trump: ShengjiTrump,
    hierarchy: &[Vec<FollowPattern>],
) -> Vec<ShengjiCard> {
    const ASSIGNMENT_LIMIT: usize = 4096;
    let Some(tier) = best_follow_tier(cards, trump, hierarchy, false) else {
        return Vec::new();
    };
    if hierarchy[tier].iter().any(|pattern| pattern.wildcards > 0) {
        return Vec::new();
    }
    let stocks = face_stocks(cards, trump);
    let mut assignments = Vec::new();
    for pattern in &hierarchy[tier] {
        let remaining = ASSIGNMENT_LIMIT.saturating_sub(assignments.len());
        if remaining == 0 {
            return Vec::new();
        }
        let (mut found, complete) = pattern_assignments(cards, trump, *pattern, false, remaining);
        if !complete {
            return Vec::new();
        }
        assignments.append(&mut found);
    }
    let Some(first) = assignments.first() else {
        return Vec::new();
    };
    let mut minimum = first.clone();
    for assignment in &assignments[1..] {
        for (minimum, selected) in minimum.iter_mut().zip(assignment) {
            *minimum = (*minimum).min(*selected);
        }
    }
    let forced = stocks
        .iter()
        .zip(minimum)
        .flat_map(|(stock, minimum)| {
            (if minimum == stock.cards.len() {
                stock.cards.as_slice()
            } else {
                &[]
            })
            .iter()
            .copied()
        })
        .collect::<HashSet<_>>();
    hand.iter()
        .copied()
        .filter(|card| forced.contains(card))
        .collect()
}
