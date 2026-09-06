use super::*;

#[derive(Clone, Debug)]
pub(super) struct StructureDemands {
    spaceships: Vec<usize>,
    titanics: Vec<usize>,
    tractors: Vec<usize>,
    quads: usize,
    triples: usize,
    pairs: usize,
    singles: usize,
}

impl StructureDemands {
    pub(super) fn from_play(play: &ShengjiClassifiedPlay) -> Self {
        let mut spaceships = Vec::new();
        let mut titanics = Vec::new();
        let mut tractors = Vec::new();
        let mut quads = 0;
        let mut triples = 0;
        let mut pairs = 0;
        let mut singles = 0;
        for component in &play.components {
            match component {
                Component::Single { .. } => singles += 1,
                Component::Pair { .. } => pairs += 1,
                Component::Triple { .. } => triples += 1,
                Component::Quad { .. } => quads += 1,
                Component::Tractor { pair_count, .. } => {
                    tractors.push(usize::from(*pair_count));
                }
                Component::Titanic { triple_count, .. } => {
                    titanics.push(usize::from(*triple_count));
                }
                Component::Spaceship { quad_count, .. } => {
                    spaceships.push(usize::from(*quad_count));
                }
            }
        }
        spaceships.sort_unstable_by(|left, right| right.cmp(left));
        titanics.sort_unstable_by(|left, right| right.cmp(left));
        tractors.sort_unstable_by(|left, right| right.cmp(left));
        Self {
            spaceships,
            titanics,
            tractors,
            quads,
            triples,
            pairs,
            singles,
        }
    }

    pub(super) fn pair_slots(&self) -> usize {
        self.spaceships.iter().sum::<usize>() * 2 + self.tractors.iter().sum::<usize>() + self.pairs
    }

    pub(super) fn tractors(&self) -> &[usize] {
        &self.tractors
    }
}

pub(super) fn satisfies_follow_structure(
    hand: &[ShengjiCard],
    proposed: &[ShengjiCard],
    demands: &StructureDemands,
    trump: ShengjiTrump,
    play_category: Category,
) -> bool {
    let mut hand_quads = quad_units(hand, trump);
    let mut required_spaceships = Vec::new();
    for demand in &demands.spaceships {
        let best = best_quad_run_at_most(&hand_quads, *demand);
        if best.len() >= 2 {
            required_spaceships.push(best.len());
            remove_quad_indices(&mut hand_quads, &best);
        }
    }
    let available_quad_count = quad_units(hand, trump).len();
    let required_quad_count =
        (demands.spaceships.iter().sum::<usize>() + demands.quads).min(available_quad_count);
    let mut proposed_quads = quad_units(proposed, trump);
    if proposed_quads.len() < required_quad_count {
        return false;
    }
    for run_len in required_spaceships {
        let run = best_quad_run_at_most(&proposed_quads, run_len);
        if run.len() < run_len {
            return false;
        }
        remove_quad_indices(&mut proposed_quads, &run);
    }

    let mut hand_triples = triple_units(hand, trump);
    let mut required_titanics = Vec::new();
    for demand in &demands.titanics {
        let best = best_triple_run_at_most(&hand_triples, *demand);
        if best.len() >= 2 {
            required_titanics.push(best.len());
            remove_triple_indices(&mut hand_triples, &best);
        }
    }
    let available_triple_count = triple_units(hand, trump).len();
    let required_triple_count =
        (demands.titanics.iter().sum::<usize>() + demands.triples).min(available_triple_count);
    let mut proposed_triples = triple_units(proposed, trump);
    if proposed_triples.len() < required_triple_count {
        return false;
    }
    for run_len in required_titanics {
        let run = best_triple_run_at_most(&proposed_triples, run_len);
        if run.len() < run_len {
            return false;
        }
        remove_triple_indices(&mut proposed_triples, &run);
    }

    let mut hand_pairs = pair_units(hand, trump);
    let mut required_runs = Vec::new();
    for demand in &demands.tractors {
        let best = best_run_at_most(&hand_pairs, *demand);
        if best.len() >= 2 {
            required_runs.push(best.len());
            remove_pair_indices(&mut hand_pairs, &best);
        }
    }
    let available_pair_count = pair_units(hand, trump).len();
    let required_pair_count = demands.pair_slots().min(available_pair_count);
    let proposed_pairs = pair_units(proposed, trump);
    if proposed_pairs.len() < required_pair_count {
        return false;
    }
    let mut proposed_remaining = proposed_pairs;
    for run_len in required_runs {
        let run = best_run_at_most(&proposed_remaining, run_len);
        if run.len() < run_len {
            return false;
        }
        remove_pair_indices(&mut proposed_remaining, &run);
    }
    proposed
        .iter()
        .all(|card| category(*card, trump) == play_category)
}

pub(super) fn competition_key(
    cards: &[ShengjiCard],
    demands: &StructureDemands,
    trump: ShengjiTrump,
    play_category: Category,
) -> Option<Vec<u8>> {
    let mut used = HashSet::new();
    let mut key = Vec::new();
    for demand in &demands.spaceships {
        let available = cards
            .iter()
            .copied()
            .filter(|card| !used.contains(card))
            .collect::<Vec<_>>();
        let quads = quad_units(&available, trump);
        let indices = highest_quad_run_exact(&quads, *demand)?;
        key.push(indices.iter().map(|index| quads[*index].strength).max()?);
        for index in indices {
            used.extend(quads[index].cards);
        }
    }
    for _ in 0..demands.quads {
        let available = cards
            .iter()
            .copied()
            .filter(|card| !used.contains(card))
            .collect::<Vec<_>>();
        let quad = quad_units(&available, trump)
            .into_iter()
            .max_by_key(|quad| quad.strength)?;
        key.push(quad.strength);
        used.extend(quad.cards);
    }
    for demand in &demands.titanics {
        let available = cards
            .iter()
            .copied()
            .filter(|card| !used.contains(card))
            .collect::<Vec<_>>();
        let triples = triple_units(&available, trump);
        let indices = highest_triple_run_exact(&triples, *demand)?;
        key.push(indices.iter().map(|index| triples[*index].strength).max()?);
        for index in indices {
            used.extend(triples[index].cards);
        }
    }
    for _ in 0..demands.triples {
        let available = cards
            .iter()
            .copied()
            .filter(|card| !used.contains(card))
            .collect::<Vec<_>>();
        let triple = triple_units(&available, trump)
            .into_iter()
            .max_by_key(|triple| triple.strength)?;
        key.push(triple.strength);
        used.extend(triple.cards);
    }
    let available = cards
        .iter()
        .copied()
        .filter(|card| !used.contains(card))
        .collect::<Vec<_>>();
    let mut pairs = pair_units(&available, trump);
    for demand in &demands.tractors {
        let indices = highest_run_exact(&pairs, *demand)?;
        key.push(indices.iter().map(|index| pairs[*index].strength).max()?);
        for index in indices.iter().rev() {
            for card in pairs[*index].cards {
                used.insert(card);
            }
        }
        remove_pair_indices(&mut pairs, &indices);
    }
    for _ in 0..demands.pairs {
        let (index, pair) = pairs
            .iter()
            .enumerate()
            .max_by_key(|(_, pair)| pair.strength)?;
        key.push(pair.strength);
        for card in pair.cards {
            used.insert(card);
        }
        pairs.remove(index);
    }
    let mut singles = cards
        .iter()
        .copied()
        .filter(|card| !used.contains(card))
        .map(|card| strength(card, trump))
        .collect::<Vec<_>>();
    if singles.len() < demands.singles {
        return None;
    }
    singles.sort_unstable_by(|left, right| right.cmp(left));
    key.extend(singles.into_iter().take(demands.singles));
    cards
        .iter()
        .all(|card| category(*card, trump) == play_category)
        .then_some(key)
}

pub(super) fn category_precedence(lead: Category, candidate: Category) -> u8 {
    match (lead, candidate) {
        (Category::Trump, Category::Trump) => 2,
        (Category::Suit(_), Category::Trump) => 3,
        (Category::Suit(lead), Category::Suit(candidate)) if lead == candidate => 2,
        _ => 0,
    }
}

pub(super) fn pair_units(cards: &[ShengjiCard], trump: ShengjiTrump) -> Vec<PairUnit> {
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
            Some(PairUnit {
                cards: [cards[0], cards[1]],
                strength: strength(cards[0], trump),
            })
        })
        .collect()
}

pub(super) fn triple_units(cards: &[ShengjiCard], trump: ShengjiTrump) -> Vec<TripleUnit> {
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
            if cards.len() < 3 {
                return None;
            }
            cards.sort_by_key(|card| card.deck());
            Some(TripleUnit {
                cards: [cards[0], cards[1], cards[2]],
                strength: strength(cards[0], trump),
            })
        })
        .collect()
}

pub(super) fn quad_units(cards: &[ShengjiCard], trump: ShengjiTrump) -> Vec<QuadUnit> {
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
            if cards.len() < 4 {
                return None;
            }
            cards.sort_by_key(|card| card.deck());
            Some(QuadUnit {
                cards: [cards[0], cards[1], cards[2], cards[3]],
                strength: strength(cards[0], trump),
            })
        })
        .collect()
}

pub(super) fn tractor_tops(pairs: &[PairUnit], length: usize) -> Vec<u8> {
    let mut tops = Vec::new();
    for pair in pairs {
        let start = pair.strength;
        if (0..length).all(|offset| {
            pairs
                .iter()
                .any(|candidate| candidate.strength == start + offset as u8)
        }) {
            tops.push(start + length as u8 - 1);
        }
    }
    tops
}

pub(super) fn titanic_tops(triples: &[TripleUnit], length: usize) -> Vec<u8> {
    let mut tops = Vec::new();
    for triple in triples {
        let start = triple.strength;
        if (0..length).all(|offset| {
            triples
                .iter()
                .any(|candidate| candidate.strength == start + offset as u8)
        }) {
            tops.push(start + length as u8 - 1);
        }
    }
    tops
}

pub(super) fn spaceship_tops(quads: &[QuadUnit], length: usize) -> Vec<u8> {
    let mut tops = Vec::new();
    for quad in quads {
        let start = quad.strength;
        if (0..length).all(|offset| {
            quads
                .iter()
                .any(|candidate| candidate.strength == start + offset as u8)
        }) {
            tops.push(start + length as u8 - 1);
        }
    }
    tops
}

pub(super) fn titanic_card_sets(
    cards: &[ShengjiCard],
    trump: ShengjiTrump,
    length: usize,
) -> Vec<Vec<ShengjiCard>> {
    let triples = triple_units(cards, trump);
    let mut candidates = Vec::new();
    for start in triples.iter().map(|triple| triple.strength) {
        collect_titanic_card_sets(
            &triples,
            start,
            length,
            0,
            &mut Vec::new(),
            &mut candidates,
            trump,
        );
    }
    candidates.sort_by_key(|cards| {
        cards
            .iter()
            .map(|card| (card.rank(), card.suit(), card.deck()))
            .collect::<Vec<_>>()
    });
    candidates.dedup();
    candidates
}

pub(super) fn collect_titanic_card_sets(
    triples: &[TripleUnit],
    start: u8,
    length: usize,
    offset: usize,
    current: &mut Vec<ShengjiCard>,
    output: &mut Vec<Vec<ShengjiCard>>,
    trump: ShengjiTrump,
) {
    if offset == length {
        let mut cards = current.clone();
        cards.sort_by_key(|card| (strength(*card, trump), card.deck()));
        output.push(cards);
        return;
    }
    for triple in triples
        .iter()
        .filter(|triple| triple.strength == start + offset as u8)
    {
        current.extend(triple.cards);
        collect_titanic_card_sets(triples, start, length, offset + 1, current, output, trump);
        current.truncate(current.len() - 3);
    }
}

pub(super) fn highest_run_exact(pairs: &[PairUnit], length: usize) -> Option<Vec<usize>> {
    let mut best: Option<Vec<usize>> = None;
    for start in pairs {
        let indices = (0..length)
            .map(|offset| {
                pairs
                    .iter()
                    .position(|pair| pair.strength == start.strength + offset as u8)
            })
            .collect::<Option<Vec<_>>>();
        if let Some(indices) = indices {
            let top = indices.iter().map(|index| pairs[*index].strength).max();
            let best_top = best
                .as_ref()
                .and_then(|best| best.iter().map(|index| pairs[*index].strength).max());
            if top > best_top {
                best = Some(indices);
            }
        }
    }
    best
}

pub(super) fn highest_triple_run_exact(
    triples: &[TripleUnit],
    length: usize,
) -> Option<Vec<usize>> {
    let strengths = triples
        .iter()
        .map(|triple| triple.strength)
        .collect::<Vec<_>>();
    highest_strength_run_exact(&strengths, length)
}

pub(super) fn highest_quad_run_exact(quads: &[QuadUnit], length: usize) -> Option<Vec<usize>> {
    let strengths = quads.iter().map(|quad| quad.strength).collect::<Vec<_>>();
    highest_strength_run_exact(&strengths, length)
}

pub(super) fn highest_strength_run_exact(strengths: &[u8], length: usize) -> Option<Vec<usize>> {
    let mut best = None::<Vec<usize>>;
    for start in strengths {
        let indices = (0..length)
            .map(|offset| {
                strengths
                    .iter()
                    .position(|strength| *strength == *start + offset as u8)
            })
            .collect::<Option<Vec<_>>>();
        if let Some(indices) = indices {
            let top = indices.iter().map(|index| strengths[*index]).max();
            let best_top = best
                .as_ref()
                .and_then(|best| best.iter().map(|index| strengths[*index]).max());
            if top > best_top {
                best = Some(indices);
            }
        }
    }
    best
}

pub(super) fn best_run_at_most(pairs: &[PairUnit], maximum: usize) -> Vec<usize> {
    for length in (2..=maximum).rev() {
        if let Some(run) = highest_run_exact(pairs, length) {
            return run;
        }
    }
    Vec::new()
}

pub(super) fn best_triple_run_at_most(triples: &[TripleUnit], maximum: usize) -> Vec<usize> {
    for length in (2..=maximum).rev() {
        if let Some(run) = highest_triple_run_exact(triples, length) {
            return run;
        }
    }
    Vec::new()
}

pub(super) fn best_quad_run_at_most(quads: &[QuadUnit], maximum: usize) -> Vec<usize> {
    for length in (2..=maximum).rev() {
        if let Some(run) = highest_quad_run_exact(quads, length) {
            return run;
        }
    }
    Vec::new()
}

pub(super) fn remove_pair_indices(pairs: &mut Vec<PairUnit>, indices: &[usize]) {
    let mut indices = indices.to_vec();
    indices.sort_unstable();
    indices.dedup();
    for index in indices.into_iter().rev() {
        pairs.remove(index);
    }
}

pub(super) fn remove_triple_indices(triples: &mut Vec<TripleUnit>, indices: &[usize]) {
    let mut indices = indices.to_vec();
    indices.sort_unstable();
    indices.dedup();
    for index in indices.into_iter().rev() {
        triples.remove(index);
    }
}

pub(super) fn remove_quad_indices(quads: &mut Vec<QuadUnit>, indices: &[usize]) {
    let mut indices = indices.to_vec();
    indices.sort_unstable();
    indices.dedup();
    for index in indices.into_iter().rev() {
        quads.remove(index);
    }
}

pub(super) fn validate_owned(cards: &[ShengjiCard], hand: &[ShengjiCard]) -> Result<(), PlayError> {
    let unique = cards.iter().copied().collect::<HashSet<_>>();
    if unique.len() != cards.len() {
        return Err(PlayError::DuplicatePhysicalCard);
    }
    if cards.iter().any(|card| !hand.contains(card)) {
        return Err(PlayError::CardsNotOwned);
    }
    Ok(())
}

pub(super) fn component_priority(component: &Component) -> (u8, usize, u8) {
    match component {
        Component::Single { strength, .. } => (0, 1, *strength),
        Component::Pair { strength, .. } => (1, 1, *strength),
        Component::Triple { strength, .. } => (2, 1, *strength),
        Component::Quad { strength, .. } => (3, 1, *strength),
        Component::Tractor {
            pair_count,
            top_strength,
            ..
        } => (4, usize::from(*pair_count), *top_strength),
        Component::Titanic {
            triple_count,
            top_strength,
            ..
        } => (5, usize::from(*triple_count), *top_strength),
        Component::Spaceship {
            quad_count,
            top_strength,
            ..
        } => (6, usize::from(*quad_count), *top_strength),
    }
}

pub(super) fn component_kind_order(component: &Component) -> u8 {
    match component {
        Component::Single { .. } => 0,
        Component::Pair { .. } => 1,
        Component::Triple { .. } => 2,
        Component::Quad { .. } => 3,
        Component::Tractor { .. } => 4,
        Component::Titanic { .. } => 5,
        Component::Spaceship { .. } => 6,
    }
}
