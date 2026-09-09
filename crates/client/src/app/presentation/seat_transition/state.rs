//! 大厅席位与开局移动演出的状态类型。

use bevy::prelude::*;
use leocard_protocol::{MatchId, PlayerId};

pub(crate) const START_GAME_SEAT_MOVE_DURATION: f32 = 0.72;

#[derive(Clone)]
pub(crate) struct LobbySeatTransitionSnapshot {
    pub player: PlayerId,
    pub center_global: Vec2,
    pub size: Vec2,
}

#[derive(Resource, Default)]
pub(crate) struct StartGameSeatTransition {
    pub match_id: Option<MatchId>,
    pub elapsed: f32,
    pub seats: Vec<LobbySeatTransitionSnapshot>,
}

impl StartGameSeatTransition {
    pub(crate) fn begin(&mut self, match_id: MatchId, seats: Vec<LobbySeatTransitionSnapshot>) {
        if seats.is_empty() {
            self.clear();
            return;
        }
        self.match_id = Some(match_id);
        self.elapsed = 0.0;
        self.seats = seats;
    }

    pub(crate) fn is_active_for(&self, match_id: MatchId) -> bool {
        self.match_id == Some(match_id) && !self.seats.is_empty()
    }

    pub(crate) fn clear(&mut self) {
        *self = Self::default();
    }
}

#[derive(Clone, Copy, Component)]
pub(crate) struct LobbySeatTransitionSource(pub PlayerId);

#[derive(Component)]
pub(crate) struct LobbySeatHover {
    pub seat: u8,
    pub amount: f32,
}

#[derive(Component)]
pub(crate) struct LobbySeatVisual(pub u8);

#[derive(Component)]
pub(crate) struct LobbyEmptySeatRing(pub u8);

#[derive(Component)]
pub(crate) struct LobbyEmptySeatLabel(pub u8);

#[derive(Clone, Copy, Component)]
pub(crate) struct GameSeatTransitionTarget(pub PlayerId);

#[derive(Clone, Copy, Component, Default)]
pub(crate) struct GameSeatTransitionPose {
    pub start_translation: Vec2,
    pub start_scale: Vec2,
    pub initialized: bool,
}
