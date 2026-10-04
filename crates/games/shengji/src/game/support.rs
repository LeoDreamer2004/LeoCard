use super::GameError;
use crate::{ShengjiCard, ShengjiPlayerId, ShengjiRuleSet, build_deck_for};
use leocard_game_common::{contains_unique_cards, validate_deck as validate_physical_deck};

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
    if !contains_unique_cards(hand, cards) {
        return Err(GameError::CardsNotOwned);
    }
    Ok(())
}

pub(super) fn validate_deck(deck: &[ShengjiCard], rules: ShengjiRuleSet) -> Result<(), GameError> {
    validate_physical_deck(deck, &build_deck_for(rules.deck_count)).map_err(GameError::from)
}
