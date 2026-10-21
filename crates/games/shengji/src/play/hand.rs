use super::{Category, Component, PlayError, category, classify_cards};
use crate::{ShengjiCard, ShengjiRank, ShengjiSuit, ShengjiTrump};
use std::collections::{BTreeMap, HashSet};

/// Finds maximal consecutive groups in a hand, preferring four-of-a-kind runs,
/// then triples, then pairs. Extra copies may supply a smaller run (8877766).
/// Faces used by a larger run are not repeated in smaller hints.
pub fn hand_sequences(
    hand: &[ShengjiCard],
    trump: ShengjiTrump,
) -> Result<Vec<Component>, PlayError> {
    if hand.iter().collect::<HashSet<_>>().len() != hand.len() {
        return Err(PlayError::DuplicatePhysicalCard);
    }
    let mut sequences = Vec::new();
    for group in [Category::Trump]
        .into_iter()
        .chain(ShengjiSuit::ALL.map(Category::Suit))
    {
        let mut faces = BTreeMap::<(Option<ShengjiSuit>, ShengjiRank), Vec<ShengjiCard>>::new();
        for card in hand.iter().filter(|card| category(**card, trump) == group) {
            faces
                .entry((card.suit(), card.rank()))
                .or_default()
                .push(*card);
        }
        for cards in faces.values_mut() {
            cards.sort_by_key(|card| card.deck());
        }
        for copies in (2..=4).rev() {
            let candidates: Vec<_> = faces
                .values()
                .filter(|cards| cards.len() >= copies)
                .flat_map(|cards| cards.iter().take(copies).copied())
                .collect();
            if candidates.is_empty() {
                continue;
            }
            for component in classify_cards(&candidates, trump)?.components {
                if matches!(
                    component,
                    Component::Tractor { .. }
                        | Component::Titanic { .. }
                        | Component::Spaceship { .. }
                ) {
                    for card in component.cards() {
                        faces.remove(&(card.suit(), card.rank()));
                    }
                    sequences.push(component);
                }
            }
        }
    }
    Ok(sequences)
}
