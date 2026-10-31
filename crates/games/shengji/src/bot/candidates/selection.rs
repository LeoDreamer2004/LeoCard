use super::{follow::responses, units};
use crate::{ShengjiCard, ShengjiClassifiedPlay, ShengjiTrump};
use std::collections::{BTreeMap, HashSet};

pub(in super::super) fn groups(hand: &[ShengjiCard]) -> Vec<Vec<ShengjiCard>> {
    let mut faces = BTreeMap::new();
    for &card in hand {
        faces
            .entry((card.suit(), card.rank()))
            .or_insert_with(Vec::new)
            .push(card);
    }
    faces
        .into_values()
        .map(|mut cards| {
            cards.sort_by_key(|card| card.deck());
            cards
        })
        .collect()
}

pub(in super::super) fn unique(choices: &mut Vec<ShengjiClassifiedPlay>) {
    let mut seen = HashSet::new();
    choices.retain(|play| seen.insert(super::super::hand::counts(&play.cards)));
}

pub(in super::super) fn plays(
    hand: &[ShengjiCard],
    lead: Option<&ShengjiClassifiedPlay>,
    trump: ShengjiTrump,
    limit: usize,
) -> Vec<ShengjiClassifiedPlay> {
    let mut choices = match lead {
        Some(lead) => responses(hand, lead, trump, limit),
        None => units(hand, trump),
    };
    unique(&mut choices);
    choices
}

/// Enumerates face multiplicities rather than combinations of physical copies.
/// Returns false if the entire space did not fit within the supplied cap.
pub(in super::super) fn combinations(
    faces: &[Vec<ShengjiCard>],
    required: usize,
    cap: usize,
    accept: &mut impl FnMut(&[ShengjiCard]),
) -> bool {
    fn visit(
        faces: &[Vec<ShengjiCard>],
        required: usize,
        selected: &mut Vec<ShengjiCard>,
        left: &mut usize,
        accept: &mut impl FnMut(&[ShengjiCard]),
    ) -> bool {
        if required == 0 {
            if *left == 0 {
                return false;
            }
            *left -= 1;
            accept(selected);
            return true;
        }
        let Some((first, rest)) = faces.split_first() else {
            return true;
        };
        let available: usize = rest.iter().map(Vec::len).sum();
        for count in required.saturating_sub(available)..=required.min(first.len()) {
            let old = selected.len();
            selected.extend_from_slice(&first[..count]);
            if !visit(rest, required - count, selected, left, accept) {
                return false;
            }
            selected.truncate(old);
        }
        true
    }
    let mut left = cap;
    visit(faces, required, &mut Vec::new(), &mut left, accept)
}
