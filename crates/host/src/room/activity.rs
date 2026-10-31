use leocard_game_common::PlayerId;
use leocard_protocol::{PlayerInteraction, PlayerInteractionKind};
use std::time::Instant;

#[derive(Clone, Debug)]
pub(super) struct RoomActivity {
    started: Instant,
}

impl Default for RoomActivity {
    fn default() -> Self {
        Self {
            started: Instant::now(),
        }
    }
}

impl RoomActivity {
    pub(super) fn interaction(
        &self,
        source: PlayerId,
        target: PlayerId,
        kind: PlayerInteractionKind,
        seed: u32,
    ) -> PlayerInteraction {
        PlayerInteraction {
            source,
            target,
            kind,
            seed,
            elapsed_millis: self.started.elapsed().as_millis() as u64,
        }
    }
}
