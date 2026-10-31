//! Recall the last completed trick using publicly broadcast actions only.

use super::DoudizhuUiState;
use crate::app::shell::UiState;
use bevy::prelude::*;
use leocard_doudizhu::DoudizhuAction;
use leocard_game_common::PlayerId;

use std::mem;

#[derive(Default)]
pub(super) struct DoudizhuTrickHistory {
    current: Vec<Option<DoudizhuAction>>,
    previous: Vec<Option<DoudizhuAction>>,
    passes: u8,
    reveal_remaining: f32,
}

impl DoudizhuTrickHistory {
    pub(super) fn observe(&mut self, player: PlayerId, action: &DoudizhuAction, player_count: u8) {
        if matches!(action, DoudizhuAction::Bid(_)) {
            return;
        }
        self.current.resize_with(usize::from(player_count), || None);
        self.current[usize::from(player.0)] = Some(action.clone());
        match action {
            DoudizhuAction::Play(_) => self.passes = 0,
            DoudizhuAction::Pass => self.passes += 1,
            DoudizhuAction::Bid(_) => unreachable!("bids do not belong to a trick"),
        }
        if self.passes == player_count - 1 {
            // A midgame join may have missed the lead; don't offer an empty history.
            if self
                .current
                .iter()
                .flatten()
                .any(|a| matches!(a, DoudizhuAction::Play(_)))
            {
                self.previous = mem::take(&mut self.current);
            } else {
                self.current.clear();
            }
            self.passes = 0;
            self.reveal_remaining = 0.0;
        }
    }

    pub(super) fn has_previous(&self) -> bool {
        !self.previous.is_empty()
    }

    pub(super) fn reveal(&mut self) {
        if self.has_previous() {
            self.reveal_remaining = 2.0;
        }
    }

    pub(super) fn revealed_actions(&self) -> Option<&[Option<DoudizhuAction>]> {
        (self.reveal_remaining > 0.0).then_some(self.previous.as_slice())
    }

    pub(super) fn advance(&mut self, delta: f32) -> bool {
        if self.reveal_remaining == 0.0 {
            return false;
        }
        self.reveal_remaining = (self.reveal_remaining - delta).max(0.0);
        self.reveal_remaining == 0.0
    }

    pub(super) fn clear(&mut self) {
        *self = Self::default();
    }
}

pub(super) fn advance_previous_trick(
    time: Res<Time>,
    mut state: ResMut<DoudizhuUiState>,
    mut ui: ResMut<UiState>,
) {
    if state.history.advance(time.delta_secs()) {
        ui.dirty = true;
    }
}
