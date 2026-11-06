use crate::app::games::GameScreenChange;
use bevy::prelude::*;
use leocard_protocol::BlackjackSnapshot;

#[derive(Resource)]
pub(crate) struct BlackjackUiState {
    pub bet: u32,
    pub guide_open: bool,
}

impl Default for BlackjackUiState {
    fn default() -> Self {
        Self {
            bet: 2,
            guide_open: false,
        }
    }
}

impl BlackjackUiState {
    pub(crate) fn reconcile(&mut self, game: &BlackjackSnapshot) {
        let available = game.available;
        if available.minimum_bet > 0 {
            self.bet = (self.bet / 2 * 2).clamp(available.minimum_bet, available.maximum_bet);
        }
    }

    pub(crate) fn clear(&mut self) {
        *self = Self::default();
    }
}

impl GameScreenChange for BlackjackSnapshot {
    fn is_transient_change(&self, after: &Self) -> bool {
        if self.seconds_remaining == after.seconds_remaining {
            return false;
        }
        let mut normalized = self.clone();
        normalized.seconds_remaining = after.seconds_remaining;
        normalized == *after
    }
}
