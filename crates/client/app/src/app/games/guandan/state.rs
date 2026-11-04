use crate::app::presentation::{CardAnimationState, CardCounterWindowState};
use bevy::prelude::*;
use leocard_guandan::GuandanCard;
use leocard_protocol::{GuandanSnapshot, MatchId};
use leocard_ui::{DraggableWindowOwner, DraggableWindowState};
use std::collections::{HashMap, HashSet};

#[derive(Resource)]
pub(super) struct GuandanUiState {
    pub selected: HashSet<GuandanCard>,
    pub card_animations: HashMap<GuandanCard, CardAnimationState>,
    pub counter: CardCounterWindowState,
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
            match_id: None,
            hand_number: 0,
        }
    }
}

impl GuandanUiState {
    pub fn reconcile(&mut self, game: &GuandanSnapshot) {
        if self.match_id != Some(game.match_id) || self.hand_number != game.hand_number {
            self.selected.clear();
            self.card_animations.clear();
            self.counter.drag.grab = None;
            self.match_id = Some(game.match_id);
            self.hand_number = game.hand_number;
        }
        self.selected.retain(|card| game.your_hand.contains(card));
        self.card_animations
            .retain(|card, _| game.your_hand.contains(card));
    }

    pub fn clear(&mut self) {
        self.selected.clear();
        self.card_animations.clear();
        self.counter.active = false;
        self.counter.drag.grab = None;
        self.match_id = None;
        self.hand_number = 0;
    }
}

impl DraggableWindowOwner for GuandanUiState {
    fn window_state(&mut self) -> &mut DraggableWindowState {
        &mut self.counter.drag
    }
}
