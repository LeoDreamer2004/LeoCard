use super::windows::PendingAction;
use leocard_game_common::PlayerId;
use leocard_protocol::{ActionClock, ActionTimingSnapshot, GameKind, MatchId};
use std::time::Duration;

#[derive(Clone, Debug, Default)]
pub(crate) struct ActionTimingState {
    now: Duration,
    scope: Option<(GameKind, MatchId)>,
    active: Vec<ActiveWindow>,
}

#[derive(Clone, Debug)]
struct ActiveWindow {
    player: PlayerId,
    key: Vec<u8>,
    started_at: Duration,
}

impl ActionTimingState {
    pub(super) fn advance(&mut self, elapsed: Duration) {
        self.now = self.now.saturating_add(elapsed);
    }

    pub(super) fn sync(&mut self, game: GameKind, match_id: MatchId, windows: Vec<PendingAction>) {
        if self.scope != Some((game, match_id)) {
            self.active.clear();
            self.scope = Some((game, match_id));
        }
        let mut active = Vec::with_capacity(windows.len());
        for window in windows {
            if active
                .iter()
                .any(|entry: &ActiveWindow| entry.player == window.player)
            {
                continue;
            }
            let started_at = self
                .active
                .iter()
                .find(|entry| entry.player == window.player && entry.key == window.key)
                .map_or(self.now, |entry| entry.started_at);
            active.push(ActiveWindow {
                player: window.player,
                key: window.key,
                started_at,
            });
        }
        self.active = active;
    }

    pub(super) fn complete(&mut self, player: PlayerId) -> Option<u64> {
        let index = self
            .active
            .iter()
            .position(|entry| entry.player == player)?;
        let entry = self.active.remove(index);
        Some(
            self.now
                .saturating_sub(entry.started_at)
                .as_millis()
                .min(u128::from(u64::MAX)) as u64,
        )
    }

    pub(super) fn clear(&mut self) {
        self.scope = None;
        self.active.clear();
    }

    pub(crate) fn snapshot(&self) -> Option<ActionTimingSnapshot> {
        let (game, match_id) = self.scope?;
        Some(ActionTimingSnapshot {
            game,
            match_id,
            clocks: self
                .active
                .iter()
                .map(|entry| ActionClock {
                    player: entry.player,
                    elapsed_millis: self
                        .now
                        .saturating_sub(entry.started_at)
                        .as_millis()
                        .min(u128::from(u64::MAX)) as u64,
                })
                .collect(),
        })
    }
}
