use super::GameError;
use crate::{ShengjiCard, ShengjiPlayerId, ShengjiRuleSet, build_deck_for};
use std::collections::HashSet;

pub(super) fn next_player(player: ShengjiPlayerId) -> ShengjiPlayerId {
    ShengjiPlayerId((player.0 + 1) % ShengjiRuleSet::PLAYER_COUNT as u8)
}

pub(super) const fn partner(player: ShengjiPlayerId) -> ShengjiPlayerId {
    ShengjiPlayerId((player.0 + 2) % ShengjiRuleSet::PLAYER_COUNT as u8)
}

pub(super) fn remove_cards(hand: &mut Vec<ShengjiCard>, cards: &[ShengjiCard]) {
    for card in cards {
        let index = hand
            .iter()
            .position(|candidate| candidate == card)
            .expect("ownership validated before removal");
        hand.remove(index);
    }
}

pub(super) fn validate_cards_owned(
    cards: &[ShengjiCard],
    hand: &[ShengjiCard],
) -> Result<(), GameError> {
    let unique = cards.iter().copied().collect::<HashSet<_>>();
    if unique.len() != cards.len() || cards.iter().any(|card| !hand.contains(card)) {
        return Err(GameError::CardsNotOwned);
    }
    Ok(())
}

pub(super) fn validate_deck(deck: &[ShengjiCard], rules: ShengjiRuleSet) -> Result<(), GameError> {
    let expected = build_deck_for(rules.deck_count);
    if deck.len() != expected.len() {
        return Err(GameError::InvalidDeckSize {
            expected: expected.len(),
            actual: deck.len(),
        });
    }
    let expected = expected.into_iter().collect::<HashSet<_>>();
    let actual = deck.iter().copied().collect::<HashSet<_>>();
    if actual.len() != deck.len() || actual != expected {
        return Err(GameError::InvalidDeckContents);
    }
    Ok(())
}
