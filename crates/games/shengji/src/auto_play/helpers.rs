use crate::{
    Component, ShengjiCard, ShengjiClassifiedPlay, ShengjiRank, ShengjiSuit, ShengjiTrump,
};
use std::cmp::Ordering;
use std::collections::{BTreeMap, HashSet};

pub(super) fn choose_short_category_fill(
    hand: &[ShengjiCard],
    same_category: &[ShengjiCard],
    required: usize,
    trump: ShengjiTrump,
) -> Vec<ShengjiCard> {
    let mut selected = same_category.to_vec();
    let selected_set = selected.iter().copied().collect::<HashSet<_>>();
    let mut fillers = hand
        .iter()
        .copied()
        .filter(|card| !selected_set.contains(card))
        .collect::<Vec<_>>();
    // 缺门时先垫任意最小副牌；副牌不足或只剩主牌时再从最小主牌开始补足。
    fillers.sort_by(|left, right| card_cmp(*left, *right, trump));
    selected.extend(fillers.into_iter().take(required - selected.len()));
    selected
}

pub(super) fn choose_structured_follow(
    same_category: &[ShengjiCard],
    lead: &ShengjiClassifiedPlay,
    trump: ShengjiTrump,
) -> Vec<ShengjiCard> {
    let required = lead.cards.len();
    let (tractor_demands, pair_slots) = structure_demands(lead);
    let all_pairs = pair_choices(same_category, trump);
    let mut obligation_pairs = all_pairs.clone();
    let mut required_runs = Vec::new();
    for maximum in tractor_demands {
        if let Some(run) = highest_run_at_most(&obligation_pairs, maximum)
            && run.len() >= 2
        {
            required_runs.push(run.len());
            remove_pair_indices(&mut obligation_pairs, &run);
        }
    }
    let required_pair_count = pair_slots.min(all_pairs.len());

    let mut available_pairs = all_pairs;
    let mut selected = Vec::with_capacity(required);
    for run_len in required_runs {
        if let Some(run) = lowest_run_exact(&available_pairs, run_len) {
            for index in &run {
                selected.extend(available_pairs[*index].cards);
            }
            remove_pair_indices(&mut available_pairs, &run);
        }
    }
    while selected.len() / 2 < required_pair_count {
        let Some((index, pair)) = available_pairs
            .iter()
            .enumerate()
            .min_by(|(_, left), (_, right)| pair_cmp(left, right))
        else {
            break;
        };
        selected.extend(pair.cards);
        available_pairs.remove(index);
    }

    let selected_set = selected.iter().copied().collect::<HashSet<_>>();
    let mut singles = same_category
        .iter()
        .copied()
        .filter(|card| !selected_set.contains(card))
        .collect::<Vec<_>>();
    singles.sort_by(|left, right| card_cmp(*left, *right, trump));
    selected.extend(singles.into_iter().take(required - selected.len()));
    selected
}

fn structure_demands(lead: &ShengjiClassifiedPlay) -> (Vec<usize>, usize) {
    let mut tractors = Vec::new();
    let mut pair_slots = 0;
    for component in &lead.components {
        match component {
            Component::Single { .. } => {}
            Component::Pair { .. } => pair_slots += 1,
            Component::Triple { .. } => pair_slots += 1,
            Component::Quad { .. } => pair_slots += 2,
            Component::Tractor { pair_count, .. } => {
                let pair_count = usize::from(*pair_count);
                tractors.push(pair_count);
                pair_slots += pair_count;
            }
            Component::Titanic { triple_count, .. } => {
                let triple_count = usize::from(*triple_count);
                tractors.push(triple_count);
                pair_slots += triple_count;
            }
            Component::Spaceship { quad_count, .. } => {
                let pair_count = usize::from(*quad_count) * 2;
                tractors.push(pair_count);
                pair_slots += pair_count;
            }
        }
    }
    tractors.sort_unstable_by(|left, right| right.cmp(left));
    (tractors, pair_slots)
}

#[derive(Clone, Debug)]
struct PairChoice {
    cards: [ShengjiCard; 2],
    strength: u8,
}

fn pair_choices(cards: &[ShengjiCard], trump: ShengjiTrump) -> Vec<PairChoice> {
    let mut faces: BTreeMap<(Option<ShengjiSuit>, ShengjiRank), Vec<ShengjiCard>> = BTreeMap::new();
    for card in cards {
        faces
            .entry((card.suit(), card.rank()))
            .or_default()
            .push(*card);
    }
    faces
        .into_values()
        .filter_map(|mut cards| {
            if cards.len() < 2 {
                return None;
            }
            cards.sort_by_key(|card| card.deck());
            Some(PairChoice {
                cards: [cards[0], cards[1]],
                strength: trump.strength(cards[0]),
            })
        })
        .collect()
}

fn highest_run_at_most(pairs: &[PairChoice], maximum: usize) -> Option<Vec<usize>> {
    (2..=maximum)
        .rev()
        .find_map(|length| highest_run_exact(pairs, length))
}

fn highest_run_exact(pairs: &[PairChoice], length: usize) -> Option<Vec<usize>> {
    pairs
        .iter()
        .filter_map(|start| run_from(pairs, start.strength, length))
        .max_by_key(|run| pairs[*run.last().unwrap()].strength)
}

fn lowest_run_exact(pairs: &[PairChoice], length: usize) -> Option<Vec<usize>> {
    pairs
        .iter()
        .filter_map(|start| run_from(pairs, start.strength, length))
        .min_by_key(|run| pairs[*run.last().unwrap()].strength)
}

fn run_from(pairs: &[PairChoice], start: u8, length: usize) -> Option<Vec<usize>> {
    (0..length)
        .map(|offset| {
            pairs
                .iter()
                .position(|pair| pair.strength == start + offset as u8)
        })
        .collect()
}

fn remove_pair_indices(pairs: &mut Vec<PairChoice>, indices: &[usize]) {
    let mut indices = indices.to_vec();
    indices.sort_unstable();
    indices.dedup();
    for index in indices.into_iter().rev() {
        pairs.remove(index);
    }
}

fn pair_cmp(left: &PairChoice, right: &PairChoice) -> Ordering {
    left.strength.cmp(&right.strength).then_with(|| {
        left.cards[0]
            .suit()
            .cmp(&right.cards[0].suit())
            .then_with(|| left.cards[0].deck().cmp(&right.cards[0].deck()))
    })
}

pub(super) fn card_cmp(left: ShengjiCard, right: ShengjiCard, trump: ShengjiTrump) -> Ordering {
    card_key(left, trump).cmp(&card_key(right, trump))
}

fn card_key(card: ShengjiCard, trump: ShengjiTrump) -> (bool, u8, u8, u8) {
    (
        trump.is_trump(card),
        trump.strength(card),
        card.suit().map_or(4, ShengjiSuit::bid_strength),
        card.deck(),
    )
}
