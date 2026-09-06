//! 大厅席位与开局移动演出的状态类型。

use super::*;

#[derive(Clone)]
pub struct LobbySeatTransitionSnapshot {
    pub player: PlayerId,
    pub center_global: Vec2,
    pub size: Vec2,
}

#[derive(Resource, Default)]
pub struct StartGameSeatTransition {
    pub match_id: Option<MatchId>,
    pub elapsed: f32,
    pub seats: Vec<LobbySeatTransitionSnapshot>,
}

impl StartGameSeatTransition {
    pub fn begin(&mut self, match_id: MatchId, seats: Vec<LobbySeatTransitionSnapshot>) {
        if seats.is_empty() {
            self.clear();
            return;
        }
        self.match_id = Some(match_id);
        self.elapsed = 0.0;
        self.seats = seats;
    }

    pub fn is_active_for(&self, match_id: MatchId) -> bool {
        self.match_id == Some(match_id) && !self.seats.is_empty()
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }
}

#[derive(Clone, Copy, Component)]
pub struct LobbySeatTransitionSource(pub PlayerId);

#[derive(Component)]
pub struct LobbySeatHover {
    pub seat: u8,
    pub amount: f32,
}

#[derive(Component)]
pub struct LobbySeatVisual(pub u8);

#[derive(Component)]
pub struct LobbyEmptySeatRing(pub u8);

#[derive(Component)]
pub struct LobbyEmptySeatLabel(pub u8);

#[derive(Clone, Copy, Component)]
pub struct GameSeatTransitionTarget(pub PlayerId);

#[derive(Clone, Copy, Component, Default)]
pub struct GameSeatTransitionPose {
    pub start_translation: Vec2,
    pub start_scale: Vec2,
    pub initialized: bool,
}
