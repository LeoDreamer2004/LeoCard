use leocard_protocol::{PlayerId, PlayerInteraction, PlayerInteractionKind};
use std::time::Instant;

#[derive(Clone, Debug)]
pub(crate) struct RoomActivity {
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
    pub(crate) fn interaction(
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
