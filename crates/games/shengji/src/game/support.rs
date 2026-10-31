use super::ShengjiGameError;
use crate::{ShengjiCard, ShengjiRuleSet, build_deck_for};
use leocard_game_common::{
    PlayerId, contains_unique_cards, validate_deck as validate_physical_deck,
};

pub(super) fn next_player(player: PlayerId, player_count: u8) -> PlayerId {
    PlayerId((player.0 + 1) % player_count)
}

pub(super) const fn partner(player: PlayerId) -> PlayerId {
    PlayerId((player.0 + 2) % ShengjiRuleSet::PLAYER_COUNT)
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
) -> Result<(), ShengjiGameError> {
    if !contains_unique_cards(hand, cards) {
        return Err(ShengjiGameError::CardsNotOwned);
    }
    Ok(())
}

pub(super) fn validate_deck(
    deck: &[ShengjiCard],
    rules: ShengjiRuleSet,
) -> Result<(), ShengjiGameError> {
    validate_physical_deck(deck, &build_deck_for(rules.deck_count)).map_err(ShengjiGameError::from)
}
