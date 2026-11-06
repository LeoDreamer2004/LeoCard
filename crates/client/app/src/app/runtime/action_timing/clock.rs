use bevy::prelude::*;
use leocard_game_common::PlayerId;
use leocard_protocol::{ActionTimingSnapshot, RoomId};
use std::time::Duration;

/// Only game portraits carry this marker; lobby ready avatars are unaffected.
#[derive(Component)]
pub(crate) struct ActionTimingLabel(pub PlayerId);

#[derive(Default, Resource)]
pub(super) struct ActionTimingClock {
    source: Option<(RoomId, u64)>,
    snapshot: Option<ActionTimingSnapshot>,
    received_at: Duration,
}

impl ActionTimingClock {
    pub(super) fn clear(&mut self) {
        self.source = None;
        self.snapshot = None;
    }

    pub(super) fn synchronize(
        &mut self,
        room: RoomId,
        serial: u64,
        snapshot: &ActionTimingSnapshot,
        now: Duration,
    ) {
        if self.source == Some((room, serial)) {
            return;
        }
        self.source = Some((room, serial));
        self.snapshot = Some(snapshot.clone());
        self.received_at = now;
    }

    pub(super) fn elapsed(&self, player: PlayerId, now: Duration) -> Option<Duration> {
        let clock = self
            .snapshot
            .as_ref()?
            .clocks
            .iter()
            .find(|clock| clock.player == player)?;
        Some(
            Duration::from_millis(clock.elapsed_millis)
                .saturating_add(now.saturating_sub(self.received_at)),
        )
    }
}
