use super::super::{
    Component, PairUnit, ShengjiCard, ShengjiClassifiedPlay, ShengjiSuit, ShengjiTrump,
    StructureDemands, best_run_at_most, category, pair_units, remove_pair_indices, strength,
    titanic_card_sets, triple_units,
};
use super::{
    forced_cards_for_hierarchy, special_follow_card_sets, special_follow_hierarchy,
    triple_follow_rank, validate_follow,
};
use std::collections::HashSet;

pub fn forced_follow_cards(
    hand: &[ShengjiCard],
    lead: &ShengjiClassifiedPlay,
    trump: ShengjiTrump,
) -> Vec<ShengjiCard> {
    let required = lead.cards.len();
    if required == 0 || hand.is_empty() {
        return Vec::new();
    }
    if hand.len() <= required {
        return hand.to_vec();
    }

    let same_category = hand
        .iter()
        .copied()
        .filter(|card| category(*card, trump) == lead.category)
        .collect::<Vec<_>>();
    if same_category.len() <= required {
        return same_category;
    }
    if let Some(hierarchy) = special_follow_hierarchy(lead) {
        return forced_cards_for_hierarchy(hand, &same_category, trump, &hierarchy);
    }

    if matches!(lead.components.as_slice(), [Component::Triple { .. }]) {
        match triple_follow_rank(&same_category) {
            2 => {
                let triples = triple_units(&same_category, trump);
                if triples.len() == 1 {
                    return triples[0].cards.to_vec();
                }
            }
            1 => {
                let pairs = pair_units(&same_category, trump);
                if pairs.len() == 1 {
                    return pairs[0].cards.to_vec();
                }
            }
            _ => {}
        }
    }
    if let [Component::Titanic { triple_count, .. }] = lead.components.as_slice() {
        let titanics = titanic_card_sets(&same_category, trump, usize::from(*triple_count));
        if let Some(first) = titanics.first() {
            let mut intersection = first.iter().copied().collect::<HashSet<_>>();
            for candidate in &titanics[1..] {
                let candidate = candidate.iter().copied().collect::<HashSet<_>>();
                intersection.retain(|card| candidate.contains(card));
            }
            return hand
                .iter()
                .copied()
                .filter(|card| intersection.contains(card))
                .collect();
        }
    }

    let demands = StructureDemands::from_play(lead);
    let all_pairs = pair_units(&same_category, trump);
    let mut obligation_pairs = all_pairs.clone();
    let mut required_runs = Vec::new();
    for demand in demands.tractors() {
        let best = best_run_at_most(&obligation_pairs, *demand);
        if best.len() >= 2 {
            required_runs.push(best.len());
            remove_pair_indices(&mut obligation_pairs, &best);
        }
    }
    let required_pair_count = demands.pair_slots().min(all_pairs.len());
    if required_pair_count == 0 {
        return Vec::new();
    }

    let mut forced = None::<HashSet<ShengjiCard>>;
    for mask in 0_u32..(1_u32 << all_pairs.len()) {
        if mask.count_ones() as usize != required_pair_count {
            continue;
        }
        let mut selected_pairs = all_pairs
            .iter()
            .enumerate()
            .filter(|(index, _)| mask & (1 << index) != 0)
            .map(|(_, pair)| pair.clone())
            .collect::<Vec<_>>();
        if !pair_selection_satisfies_runs(&mut selected_pairs, &required_runs) {
            continue;
        }
        let selected = all_pairs
            .iter()
            .enumerate()
            .filter(|(index, _)| mask & (1 << index) != 0)
            .flat_map(|(_, pair)| pair.cards)
            .collect::<HashSet<_>>();
        match &mut forced {
            Some(forced) => forced.retain(|card| selected.contains(card)),
            None => forced = Some(selected),
        }
    }

    let forced = forced.unwrap_or_default();
    hand.iter()
        .copied()
        .filter(|card| forced.contains(card))
        .collect()
}

pub fn follow_suggestions(
    hand: &[ShengjiCard],
    lead: &ShengjiClassifiedPlay,
    trump: ShengjiTrump,
    limit: usize,
) -> Vec<ShengjiClassifiedPlay> {
    const SEARCH_BUDGET: usize = 200_000;

    let required = lead.cards.len();
    if limit == 0 || required == 0 || hand.len() < required {
        return Vec::new();
    }
    let mut same_category = hand
        .iter()
        .copied()
        .filter(|card| category(*card, trump) == lead.category)
        .collect::<Vec<_>>();
    let mut other = hand
        .iter()
        .copied()
        .filter(|card| category(*card, trump) != lead.category)
        .collect::<Vec<_>>();
    let card_key = |card: &ShengjiCard| {
        (
            trump.is_trump(*card),
            strength(*card, trump),
            card.suit().map_or(4, ShengjiSuit::bid_strength),
            card.deck(),
        )
    };
    same_category.sort_by_key(card_key);
    other.sort_by_key(card_key);

    if same_category.len() >= required
        && let Some(hierarchy) = special_follow_hierarchy(lead)
    {
        let candidates = special_follow_card_sets(&same_category, trump, &hierarchy, limit);
        let suggestions = candidates
            .into_iter()
            .filter_map(|cards| validate_follow(hand, &cards, lead, trump).ok())
            .take(limit)
            .collect::<Vec<_>>();
        if !suggestions.is_empty() {
            return suggestions;
        }
    }

    let (fixed, choices, choose_count) = if same_category.len() >= required {
        (Vec::new(), same_category, required)
    } else {
        let choose_count = required - same_category.len();
        (same_category, other, choose_count)
    };
    let mut suggestions = Vec::new();
    let mut search = CombinationSearch::new(&choices, SEARCH_BUDGET);
    search.visit(0, choose_count, &mut |chosen| {
        let mut cards = fixed.clone();
        cards.extend_from_slice(chosen);
        if let Ok(play) = validate_follow(hand, &cards, lead, trump) {
            suggestions.push(play);
        }
        suggestions.len() >= limit
    });
    suggestions
}

struct CombinationSearch<'a> {
    choices: &'a [ShengjiCard],
    current: Vec<ShengjiCard>,
    visited: usize,
    budget: usize,
}

impl<'a> CombinationSearch<'a> {
    fn new(choices: &'a [ShengjiCard], budget: usize) -> Self {
        Self {
            choices,
            current: Vec::new(),
            visited: 0,
            budget,
        }
    }

    fn visit(
        &mut self,
        start: usize,
        remaining: usize,
        accept: &mut impl FnMut(&[ShengjiCard]) -> bool,
    ) -> bool {
        if self.visited >= self.budget {
            return true;
        }
        if remaining == 0 {
            self.visited += 1;
            return accept(&self.current);
        }
        if self.choices.len().saturating_sub(start) < remaining {
            return false;
        }
        let last_start = self.choices.len() - remaining;
        for index in start..=last_start {
            self.current.push(self.choices[index]);
            if self.visit(index + 1, remaining - 1, accept) {
                self.current.pop();
                return true;
            }
            self.current.pop();
        }
        false
    }
}

fn pair_selection_satisfies_runs(
    selected_pairs: &mut Vec<PairUnit>,
    required_runs: &[usize],
) -> bool {
    for run_len in required_runs {
        let run = best_run_at_most(selected_pairs, *run_len);
        if run.len() < *run_len {
            return false;
        }
        remove_pair_indices(selected_pairs, &run);
    }
    true
}
