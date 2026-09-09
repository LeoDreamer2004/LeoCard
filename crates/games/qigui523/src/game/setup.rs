use super::{
    GameError, GameState, Phase, PlayerState, QiGuiPlayerId, TrickState, find_starting_card,
    sort_hands, validate_deck,
};
#[cfg(feature = "developer")]
use crate::build_deck;
use crate::{QiGuiCard, QiGuiRuleSet};
#[cfg(feature = "developer")]
use std::collections::HashSet;
use std::collections::VecDeque;

impl GameState {
    pub fn new_with_deck(rules: QiGuiRuleSet, deck: Vec<QiGuiCard>) -> Result<Self, GameError> {
        let rules = rules.validate()?;
        validate_deck(rules.deck_count, &deck)?;
        Ok(Self::deal_validated_deck(rules, deck))
    }

    #[cfg(feature = "developer")]
    pub fn new_with_development_deck(
        rules: QiGuiRuleSet,
        deck: Vec<QiGuiCard>,
    ) -> Result<Self, GameError> {
        let rules = rules.validate()?;
        let expected = usize::from(rules.player_count) * usize::from(rules.hand_size);
        if deck.len() != expected {
            return Err(GameError::InvalidDeckSize {
                expected,
                actual: deck.len(),
            });
        }
        let full_deck: HashSet<_> = build_deck(rules.deck_count).into_iter().collect();
        let actual: HashSet<_> = deck.iter().copied().collect();
        if actual.len() != deck.len() || !actual.is_subset(&full_deck) {
            return Err(GameError::InvalidDeckContents);
        }
        Ok(Self::deal_validated_deck(rules, deck))
    }

    fn deal_validated_deck(rules: QiGuiRuleSet, deck: Vec<QiGuiCard>) -> Self {
        let mut draw_pile = VecDeque::from(deck);
        let mut players: Vec<_> = (0..usize::from(rules.player_count))
            .map(|index| PlayerState {
                id: QiGuiPlayerId(index),
                hand: Vec::with_capacity(usize::from(rules.hand_size)),
                score: 0,
            })
            .collect();
        let mut deal_order =
            Vec::with_capacity(usize::from(rules.player_count) * usize::from(rules.hand_size));
        for _ in 0..rules.hand_size {
            for player in &mut players {
                let card = draw_pile
                    .pop_front()
                    .expect("QiGuiRuleSet::validate ensured enough cards");
                player.hand.push(card);
                deal_order.push((player.id, card));
            }
        }
        let starting_card = find_starting_card(&deal_order);
        sort_hands(&mut players);
        let trick = Some(TrickState::new(starting_card.player));
        Self {
            rules,
            players,
            draw_pile,
            starting_card,
            trick,
            phase: Phase::Playing,
        }
    }
}
