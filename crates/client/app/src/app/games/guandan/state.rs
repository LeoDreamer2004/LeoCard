use crate::app::presentation::{CardAnimationState, CardCounterWindowState};
use bevy::prelude::*;
use leocard_game_common::PlayerId;
use leocard_guandan::{
    GuandanAction, GuandanCard, GuandanPhase, GuandanPlay, GuandanTributeAction, can_offer_tribute,
    can_return_tribute, hint_options,
};
use leocard_protocol::{GuandanSnapshot, MatchId};
use leocard_ui::{DraggableWindowOwner, DraggableWindowState};
use std::collections::{HashMap, HashSet};

#[derive(Resource)]
pub(super) struct GuandanUiState {
    pub selected: HashSet<GuandanCard>,
    pub card_animations: HashMap<GuandanCard, CardAnimationState>,
    pub counter: CardCounterWindowState,
    pub previous_reveal: f32,
    pub pending_deal: bool,
    pub effect_sequence: Option<u128>,
    pub tribute_flights: Vec<(PlayerId, PlayerId)>,
    pub hints: GuandanHints,
    match_id: Option<MatchId>,
    hand_number: u32,
}

impl Default for GuandanUiState {
    fn default() -> Self {
        let mut counter = CardCounterWindowState::default();
        if cfg!(target_os = "android") {
            counter.drag.position.y = 34.0;
        }
        Self {
            selected: HashSet::new(),
            card_animations: HashMap::new(),
            counter,
            previous_reveal: 0.0,
            pending_deal: false,
            effect_sequence: None,
            tribute_flights: Vec::new(),
            hints: GuandanHints::default(),
            match_id: None,
            hand_number: 0,
        }
    }
}

impl GuandanUiState {
    pub fn reconcile(&mut self, game: &GuandanSnapshot) -> bool {
        let changed_hand =
            self.match_id != Some(game.match_id) || self.hand_number != game.hand_number;
        if changed_hand {
            let new_hand = self.match_id == Some(game.match_id);
            self.reset_hand();
            self.pending_deal = new_hand
                || (game.hand_number == 1
                    && game.your_hand.len() == 27
                    && game.played_cards.is_empty());
            self.match_id = Some(game.match_id);
            self.hand_number = game.hand_number;
        }
        if !matches!(game.phase, GuandanPhase::Playing) {
            self.previous_reveal = 0.0;
        }
        if local_turn(game) && matches!(game.phase, GuandanPhase::Playing) {
            self.hints.sync(game);
        } else {
            self.hints.clear();
        }
        self.selected.retain(|card| game.your_hand.contains(card));
        self.card_animations
            .retain(|card, _| game.your_hand.contains(card));
        changed_hand
    }

    pub fn is_dealing(&self) -> bool {
        self.card_animations
            .values()
            .any(|animation| animation.dealing)
    }

    fn reset_hand(&mut self) {
        self.selected.clear();
        self.card_animations.clear();
        self.previous_reveal = 0.0;
        self.pending_deal = false;
        self.effect_sequence = None;
        self.tribute_flights.clear();
        self.hints.clear();
        self.counter.drag.grab = None;
    }

    pub fn clear(&mut self) {
        if self.match_id.is_none() {
            return;
        }
        self.reset_hand();
        self.counter.active = false;
        self.match_id = None;
        self.hand_number = 0;
    }
}

impl DraggableWindowOwner for GuandanUiState {
    fn window_state(&mut self) -> &mut DraggableWindowState {
        &mut self.counter.drag
    }
}

#[derive(Default)]
pub(super) struct GuandanHints {
    choices: Vec<GuandanPlay>,
    hand: Vec<GuandanCard>,
    target: Option<GuandanPlay>,
    next: usize,
}

impl GuandanHints {
    fn sync(&mut self, game: &GuandanSnapshot) {
        let target = game.table.as_ref().map(|(_, play)| play);
        if self.hand != game.your_hand || self.target.as_ref() != target {
            self.choices = hint_options(&game.your_hand, game.level, target);
            self.hand.clone_from(&game.your_hand);
            self.target = target.cloned();
            self.next = 0;
        }
    }

    fn clear(&mut self) {
        self.choices.clear();
        self.hand.clear();
        self.target = None;
        self.next = 0;
    }

    pub(super) fn is_empty(&self) -> bool {
        self.choices.is_empty()
    }

    pub(super) fn next_cards(&mut self) -> Option<&[GuandanCard]> {
        let hint = self.choices.get(self.next)?;
        self.next = (self.next + 1) % self.choices.len();
        Some(&hint.cards)
    }
}

pub(super) fn local_turn(game: &GuandanSnapshot) -> bool {
    game.result.is_none()
        && game.current == Some(game.you)
        && game
            .players
            .iter()
            .any(|player| player.player.id == game.you && player.connected && !player.auto_play)
}

pub(super) fn tribute_choice(game: &GuandanSnapshot, card: GuandanCard) -> Option<GuandanAction> {
    match game.tribute_action? {
        GuandanTributeAction::Tribute => can_offer_tribute(&game.your_hand, card, game.level)
            .then_some(GuandanAction::Tribute(card)),
        GuandanTributeAction::ReturnTribute => {
            can_return_tribute(&game.your_hand, card, game.received_tribute?, game.level)
                .then_some(GuandanAction::ReturnTribute(card))
        }
    }
}
