use crate::{DoudizhuAction, DoudizhuCard, DoudizhuPlay, DoudizhuRuleSet, classify};
use std::collections::BTreeSet;
use std::ops::Range;

pub(crate) fn legal_actions(
    hand: &[DoudizhuCard],
    table: Option<&DoudizhuPlay>,
    rules: DoudizhuRuleSet,
) -> Vec<DoudizhuAction> {
    let mut groups: [Vec<DoudizhuCard>; 15] = Default::default();
    for &card in hand {
        groups[usize::from(card.rank())].push(card);
    }
    for group in &mut groups {
        group.sort_unstable();
    }
    let mut candidates = BTreeSet::new();
    let mut add = |mut cards: Vec<DoudizhuCard>| {
        cards.sort_unstable();
        if classify(&cards, rules).is_some_and(|play| table.is_none_or(|last| play.beats(last))) {
            candidates.insert(cards);
        }
    };
    for (rank, group) in groups.iter().enumerate() {
        for width in 1..=group.len() {
            add(group[..width].to_vec());
        }
        for (body, wings) in [(3, 1), (4, 2)] {
            if group.len() >= body {
                for wing_width in 1..=2 {
                    attach(
                        &groups,
                        rank..rank + 1,
                        group[..body].to_vec(),
                        wing_width,
                        wings,
                        &mut add,
                    );
                }
            }
        }
    }
    if !groups[13].is_empty() && !groups[14].is_empty() {
        add(vec![groups[13][0], groups[14][0]]);
    }
    for (width, minimum) in [(1, 5), (2, 3), (3, 2)] {
        for start in 0..12 {
            for end in start + minimum..=12 {
                if groups[start..end].iter().any(|group| group.len() < width) {
                    break;
                }
                let body: Vec<_> = groups[start..end]
                    .iter()
                    .flat_map(|group| group[..width].iter().copied())
                    .collect();
                add(body.clone());
                if width == 3 {
                    for wing_width in 1..=2 {
                        attach(
                            &groups,
                            start..end,
                            body.clone(),
                            wing_width,
                            end - start,
                            &mut add,
                        );
                    }
                }
            }
        }
    }
    let mut actions: Vec<_> = candidates.into_iter().map(DoudizhuAction::Play).collect();
    if table.is_some() {
        actions.push(DoudizhuAction::Pass);
    }
    actions
}

fn attach(
    groups: &[Vec<DoudizhuCard>; 15],
    body: Range<usize>,
    mut cards: Vec<DoudizhuCard>,
    width: usize,
    count: usize,
    add: &mut impl FnMut(Vec<DoudizhuCard>),
) {
    let available: Vec<_> = groups
        .iter()
        .enumerate()
        .filter(|(rank, group)| !body.contains(rank) && group.len() >= width)
        .map(|(_, group)| group.as_slice())
        .collect();
    wings(&available, 0, width, count, &mut cards, add);
}

fn wings(
    groups: &[&[DoudizhuCard]],
    index: usize,
    width: usize,
    needed: usize,
    cards: &mut Vec<DoudizhuCard>,
    add: &mut impl FnMut(Vec<DoudizhuCard>),
) {
    if needed == 0 {
        add(cards.clone());
        return;
    }
    let Some(group) = groups.get(index) else {
        return;
    };
    let maximum = if width == 1 {
        group.len().min(needed)
    } else {
        1
    };
    for take in 0..=maximum {
        let length = cards.len();
        cards.extend_from_slice(&group[..take * width]);
        wings(groups, index + 1, width, needed - take, cards, add);
        cards.truncate(length);
    }
}
