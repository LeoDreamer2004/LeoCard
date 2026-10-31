use super::history::DoudizhuTrickHistory;
use crate::app::presentation::{CardAnimationState, CardCounterWindowState, Observed};
use bevy::prelude::*;
use leocard_doudizhu::{Card, Phase};
use leocard_protocol::{DoudizhuSnapshot, MatchId};
use leocard_ui::{DraggableWindowOwner, DraggableWindowState};
use std::cmp::Reverse;
use std::collections::{HashMap, HashSet};

#[derive(Default, Resource)]
pub(crate) struct DoudizhuUiState {
    pub mode_menu_open: bool,
    pub selected: HashSet<Card>,
    pub kitty_open: bool,
    pub counter: CardCounterWindowState,
    pub card_animations: HashMap<Card, CardAnimationState>,
    pub pending_deal: Option<MatchId>,
    pub(super) history: DoudizhuTrickHistory,
    observed_hand: Observed<MatchId, Vec<Card>>,
    match_id: Option<MatchId>,
    hand_number: u8,
}

impl DoudizhuUiState {
    pub fn reconcile(&mut self, game: &DoudizhuSnapshot) {
        if self.match_id != Some(game.match_id) || self.hand_number != game.progress.hand_number {
            if self.match_id.is_some() {
                self.pending_deal = Some(game.match_id);
            }
            self.selected.clear();
            self.card_animations.clear();
            self.observed_hand.clear();
            self.kitty_open = false;
            self.history.clear();
            self.match_id = Some(game.match_id);
            self.hand_number = game.progress.hand_number;
        }
        self.selected.retain(|card| game.your_hand.contains(card));
        self.card_animations
            .retain(|card, _| game.your_hand.contains(card));
        if game.kitty.is_empty() {
            self.kitty_open = false;
        }
        if game.phase != Phase::Playing {
            self.history.clear();
        }
    }

    pub fn clear(&mut self) {
        self.selected.clear();
        self.card_animations.clear();
        self.match_id = None;
        self.hand_number = 0;
        self.kitty_open = false;
        self.pending_deal = None;
        self.observed_hand.clear();
        self.counter.active = false;
        self.history.clear();
    }

    pub(super) fn observe_dealt_hand(
        &mut self,
        match_id: MatchId,
        hand: &[Card],
    ) -> Vec<(Card, f32)> {
        let first_snapshot = self.observed_hand.observe(match_id);
        let full_deal = self.pending_deal == Some(match_id);
        if full_deal {
            self.pending_deal = None;
            self.observed_hand.state.clear();
            self.card_animations.clear();
            self.selected.clear();
        }
        let mut new_cards: Vec<_> = hand
            .iter()
            .copied()
            .filter(|card| !self.observed_hand.state.contains(card))
            .collect();
        new_cards.sort_by_key(|card| Reverse((card.rank(), card.0)));
        let animate = full_deal || !first_snapshot;
        let base_delay = self
            .card_animations
            .values()
            .filter(|animation| animation.dealing)
            .map(|animation| (0.20 - animation.deal_elapsed).max(0.0))
            .fold(0.0, f32::max);
        let mut sounds = Vec::new();
        for (index, card) in new_cards.into_iter().enumerate() {
            let delay = base_delay + index as f32 * 0.045;
            self.card_animations.insert(
                card,
                CardAnimationState {
                    deal_elapsed: if animate { -delay } else { 0.20 },
                    dealing: animate,
                    ..default()
                },
            );
            if animate {
                sounds.push((card, delay));
            }
        }
        self.observed_hand.state = hand.to_vec();
        self.card_animations.retain(|card, _| hand.contains(card));
        sounds
    }
}

impl DraggableWindowOwner for DoudizhuUiState {
    fn window_state(&mut self) -> &mut DraggableWindowState {
        &mut self.counter.drag
    }
}
