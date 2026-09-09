use super::{GameError, PlayerState, QiGuiPlayerId, StartingCard};
use crate::{QiGuiCard, build_deck};
use std::collections::HashSet;

pub(super) fn validate_deck(deck_count: u8, deck: &[QiGuiCard]) -> Result<(), GameError> {
    let expected_deck = build_deck(deck_count);
    if deck.len() != expected_deck.len() {
        return Err(GameError::InvalidDeckSize {
            expected: expected_deck.len(),
            actual: deck.len(),
        });
    }
    let expected: HashSet<_> = expected_deck.into_iter().collect();
    let actual: HashSet<_> = deck.iter().copied().collect();
    if actual.len() != deck.len() || actual != expected {
        return Err(GameError::InvalidDeckContents);
    }
    Ok(())
}

pub(super) fn find_starting_card(deal_order: &[(QiGuiPlayerId, QiGuiCard)]) -> StartingCard {
    let minimum_strength = deal_order
        .iter()
        .map(|(_, card)| card.semantic_strength())
        .min()
        .expect("every player receives at least one card");
    deal_order
        .iter()
        .copied()
        .find(|(_, card)| card.semantic_strength() == minimum_strength)
        .map(|(player, card)| StartingCard { player, card })
        .expect("minimum strength came from the deal order")
}

pub(super) fn remove_cards(hand: &mut Vec<QiGuiCard>, cards: &[QiGuiCard]) {
    for card in cards {
        let index = hand
            .iter()
            .position(|candidate| candidate == card)
            .expect("cards were checked before removal");
        hand.remove(index);
    }
}

pub(super) fn sort_hands(players: &mut [PlayerState]) {
    for player in players {
        player.hand.sort_by(QiGuiCard::display_cmp);
    }
}
