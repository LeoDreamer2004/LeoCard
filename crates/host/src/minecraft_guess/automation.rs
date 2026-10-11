use super::MinecraftGuessSession;
use crate::{Delivery, lifecycle::HostedGameLifecycle};
use leocard_minecraft_guess::{Catalogue, GameState};
use std::time::Duration;

const AUTOMATIC_ACTION_DELAY: Duration = Duration::from_secs(2);

#[derive(Clone, Debug, Default)]
pub(super) struct GuessTurnClock {
    elapsed: Duration,
    automatic_wait: Duration,
}

impl GuessTurnClock {
    fn remaining(&self, limit: u16) -> Option<u16> {
        if limit == 0 {
            return None;
        }
        let remaining = Duration::from_secs(u64::from(limit)).saturating_sub(self.elapsed);
        Some((remaining.as_secs() + u64::from(remaining.subsec_nanos() != 0)) as u16)
    }

    fn advance(&mut self, elapsed: Duration, automatic: bool) {
        self.elapsed = self.elapsed.saturating_add(elapsed);
        self.automatic_wait = if automatic {
            self.automatic_wait.saturating_add(elapsed)
        } else {
            Duration::ZERO
        };
    }
}

impl MinecraftGuessSession {
    pub(super) fn guess_seconds_remaining(&self) -> Option<u16> {
        self.game.as_ref().and_then(GameState::current_player)?;
        self.turn_clock.remaining(self.rules.guess_seconds)
    }

    pub fn advance_time(&mut self, elapsed: Duration) -> Vec<Delivery> {
        let Some(current) = self.game.as_ref().and_then(GameState::current_player) else {
            return Vec::new();
        };
        let before = self.guess_seconds_remaining();
        let player = &self.room.players[usize::from(current)];
        let automatic = !player.connected || player.is_bot || player.left;
        self.turn_clock.advance(elapsed, automatic);
        let expired = self.guess_seconds_remaining() == Some(0);
        if expired || (automatic && self.turn_clock.automatic_wait >= AUTOMATIC_ACTION_DELAY) {
            if player.is_bot && !player.left {
                self.guess_for_bot(current);
            } else {
                self.game
                    .as_mut()
                    .expect("an active turn has a game")
                    .skip(current)
                    .expect("current player can skip");
            }
            let mut deliveries = self.after_action();
            deliveries.extend(self.broadcast_game(None));
            return deliveries;
        }
        if self.guess_seconds_remaining() != before {
            self.bump_revision();
            self.broadcast_game(None)
        } else {
            Vec::new()
        }
    }

    fn guess_for_bot(&mut self, current: u8) {
        let game = self.game.as_mut().expect("an active turn has a game");
        // Bots use only public feedback; never peek at the chosen answer.
        let view = game.view();
        let candidates: Vec<_> = Catalogue::bundled()
            .search(view.kind, "")
            .filter(|entry| {
                entry.name.chars().count() == usize::from(view.character_count)
                    && view
                        .guesses
                        .iter()
                        .all(|guess| guess.result.accepts_candidate(entry))
            })
            .collect();
        if let Some(entry) = candidates.get(fastrand::usize(..candidates.len().max(1))) {
            game.guess(current, &entry.id)
                .expect("a public candidate is valid");
        } else {
            game.skip(current).expect("current player can skip");
        }
    }
}
