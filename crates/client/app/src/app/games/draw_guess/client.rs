use crate::app::runtime::GameProjection;
use bevy::prelude::*;
use leocard_draw_guess::{CanvasDocument, DrawAction};
use leocard_protocol::{DrawGuessEvent, GameEvent, GameSnapshot, MatchId, ServerEvent};
use std::collections::VecDeque;

#[derive(Resource, Default)]
pub(super) struct DrawGuessClient {
    pub key: Option<(MatchId, u32)>,
    pub document: CanvasDocument,
    pub pending: VecDeque<DrawAction>,
    pub needs_snapshot: bool,
    pub snapshot_requested: bool,
    pub clock_remaining_ms: u32,
    pub clock_serial: u64,
}

impl DrawGuessClient {
    pub fn predicted(&self) -> CanvasDocument {
        let mut document = self.document.clone();
        for action in &self.pending {
            if document.apply(action).is_err() {
                break;
            }
        }
        document
    }

    pub fn predict(&mut self, action: DrawAction) -> bool {
        let mut predicted = self.predicted();
        if (self.pending.len() >= 128 && !matches!(action, DrawAction::End | DrawAction::Undo))
            || predicted.apply(&action).is_err()
        {
            return false;
        }
        self.pending.push_back(action);
        true
    }

    fn apply_delta(&mut self, event: &DrawGuessEvent) {
        if self.key != Some((event.match_id, event.turn))
            || event.revision <= self.document.revision()
        {
            return;
        }
        if event.revision != self.document.revision() + 1
            || self.document.apply(&event.action).is_err()
        {
            self.needs_snapshot = true;
            return;
        }
        if self.pending.front() == Some(&event.action) {
            self.pending.pop_front();
        }
    }
}

impl GameProjection for DrawGuessClient {
    fn apply(&mut self, update: &ServerEvent) {
        match update {
            ServerEvent::GameSnapshot(GameSnapshot::DrawGuess(game)) => {
                let key = Some((game.match_id, game.state.turn));
                if self.key != key || self.needs_snapshot {
                    self.pending.clear();
                }
                self.key = key;
                self.clock_remaining_ms = game.state.remaining_ms;
                self.clock_serial = self.clock_serial.wrapping_add(1);
                if game.state.canvas.revision() >= self.document.revision()
                    || self.pending.is_empty()
                {
                    self.document = game.state.canvas.clone();
                }
                self.needs_snapshot = false;
                self.snapshot_requested = false;
            }
            ServerEvent::GameEvent(GameEvent::DrawGuess(event)) => self.apply_delta(event),
            ServerEvent::Rejected { .. } => self.pending.clear(),
            ServerEvent::GameSnapshot(_)
            | ServerEvent::LobbySnapshot(_)
            | ServerEvent::LeftRoom
            | ServerEvent::RoomClosed
            | ServerEvent::KickedFromRoom => *self = Self::default(),
            _ => {}
        }
    }
}
