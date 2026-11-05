//! Preserve network boundaries: events animate only with their following snapshot.
//!
//! A frame may end between any two messages, or contain several snapshots and
//! events. Flattening just the events against the latest model can apply future
//! actions to old balances, which authoritative reconciliation would undo.

use crate::app::runtime::GameProjection;
use bevy::prelude::*;
use leocard_game_common::PlayerId;
use leocard_protocol::{GameEvent, GameSnapshot, MatchId, ServerEvent, TexasEvent, TexasSnapshot};
use std::mem;

#[derive(Default, Resource)]
pub(crate) struct TexasChipUpdates {
    key: Option<(MatchId, PlayerId)>,
    pending: Vec<TexasEvent>,
    ready: Vec<(TexasSnapshot, Vec<TexasEvent>)>,
}

impl TexasChipUpdates {
    pub(super) fn take(&mut self) -> Vec<(TexasSnapshot, Vec<TexasEvent>)> {
        mem::take(&mut self.ready)
    }
}

impl GameProjection for TexasChipUpdates {
    fn apply(&mut self, update: &ServerEvent) {
        match update {
            ServerEvent::GameSnapshot(GameSnapshot::Texas(game)) => {
                let key = (game.match_id, game.you);
                if self.key != Some(key) {
                    *self = Self::default();
                    self.key = Some(key);
                }
                self.ready
                    .push((game.clone(), mem::take(&mut self.pending)));
            }
            ServerEvent::GameEvent(GameEvent::Texas(event)) if self.key.is_some() => {
                self.pending.push(event.clone());
            }
            ServerEvent::GameSnapshot(_)
            | ServerEvent::LobbySnapshot(_)
            | ServerEvent::RoomClosed
            | ServerEvent::LeftRoom
            | ServerEvent::KickedFromRoom => *self = Self::default(),
            _ => {}
        }
    }
}
