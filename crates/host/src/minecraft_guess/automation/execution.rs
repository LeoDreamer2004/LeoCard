use super::super::MinecraftGuessSession;
use super::{bot, fallback};
use crate::Delivery;
use crate::lifecycle::HostedGameLifecycle;
use leocard_minecraft_guess::GameState;
use std::time::Duration;

const AUTOMATIC_ACTION_DELAY: Duration = Duration::from_secs(2);

impl MinecraftGuessSession {
    pub(in super::super) fn guess_seconds_remaining(&self) -> Option<u16> {
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
        if expired || (automatic && self.turn_clock.automatic_ready(AUTOMATIC_ACTION_DELAY)) {
            if player.is_bot && !player.left {
                bot::Policy::act(
                    self.game.as_mut().expect("an active turn has a game"),
                    current,
                );
            } else {
                fallback::Policy::act(
                    self.game.as_mut().expect("an active turn has a game"),
                    current,
                );
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
}
