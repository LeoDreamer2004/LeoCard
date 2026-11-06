use leocard_game_common::PlayerId;
use leocard_protocol::{ActionClock, ActionTimingSnapshot, GameKind, MatchId};
use std::fmt::Debug;
use std::time::Duration;

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct PendingAction {
    pub player: PlayerId,
    /// Process-local equality token, never serialized or persisted.
    key: Vec<u8>,
}

impl PendingAction {
    pub(crate) fn for_players(
        players: impl IntoIterator<Item = PlayerId>,
        context: impl Debug,
    ) -> Vec<Self> {
        players
            .into_iter()
            .map(|player| Self {
                player,
                key: format!("{:?}", (player, &context)).into_bytes(),
            })
            .collect()
    }
}

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
    pub(crate) fn advance(&mut self, elapsed: Duration) {
        self.now = self.now.saturating_add(elapsed);
    }

    pub(crate) fn sync(&mut self, game: GameKind, match_id: MatchId, windows: Vec<PendingAction>) {
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

    pub(crate) fn complete(&mut self, player: PlayerId) -> Option<u64> {
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

    pub(crate) fn clear(&mut self) {
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
