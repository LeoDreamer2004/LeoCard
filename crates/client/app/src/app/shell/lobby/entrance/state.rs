use crate::app::runtime::GameProjection;
use bevy::prelude::*;
use leocard_game_common::PlayerId;
use leocard_protocol::{LobbyEntranceView, ServerEvent};

#[derive(Clone, Copy)]
pub(super) struct EntranceAnimation {
    pub sequence: u64,
    pub player: PlayerId,
    pub elapsed: f32,
    fresh: bool,
}

#[derive(Resource, Default)]
pub(crate) struct LobbyEntranceState {
    pub(super) active: Option<EntranceAnimation>,
    seen: Option<u64>,
}

impl LobbyEntranceState {
    pub(crate) fn active(&self) -> bool {
        self.active.is_some()
    }

    pub(super) fn advance(&mut self, delta: f32) {
        if let Some(active) = &mut self.active {
            // 首帧 delta 可能包含入场前的后台休眠，不能直接跳过整段演出。
            if active.fresh {
                active.fresh = false;
                return;
            }
            active.elapsed += delta;
            if active.elapsed >= f32::from(LobbyEntranceView::DURATION_MILLIS) / 1000.0 {
                self.active = None;
            }
        }
    }
}

impl GameProjection for LobbyEntranceState {
    fn apply(&mut self, event: &ServerEvent) {
        match event {
            ServerEvent::LobbySnapshot(lobby) => {
                let Some(entrance) = lobby.entrance else {
                    self.active = None;
                    return;
                };
                let elapsed = f32::from(entrance.elapsed_millis) / 1000.0;
                if self.seen != Some(entrance.sequence) {
                    self.seen = Some(entrance.sequence);
                    self.active = Some(EntranceAnimation {
                        sequence: entrance.sequence,
                        player: entrance.player,
                        elapsed,
                        fresh: true,
                    });
                } else if let Some(active) = &mut self.active {
                    active.elapsed = active.elapsed.max(elapsed);
                }
            }
            ServerEvent::GameSnapshot(_) => self.active = None,
            ServerEvent::LeftRoom | ServerEvent::RoomClosed => *self = Self::default(),
            _ => {}
        }
    }
}
