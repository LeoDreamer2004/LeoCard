use super::super::McGuessSession;
use crate::Delivery;
use crate::lifecycle::HostedGameLifecycle;
use std::time::Duration;

const BOT_INTERVAL: Duration = Duration::from_secs(8);

impl McGuessSession {
    pub(super) fn advance_race(&mut self, elapsed: Duration) -> Vec<Delivery> {
        let before = self.guess_seconds_remaining();
        self.clock.advance(elapsed, true);
        // The public deadline takes priority over automatic guesses at the same tick.
        if self.guess_seconds_remaining() == Some(0) {
            self.game
                .as_mut()
                .expect("active race")
                .expire()
                .expect("active race can expire");
            let mut deliveries = self.after_action();
            deliveries.extend(self.broadcast_game(None));
            return deliveries;
        }
        let mut deliveries = Vec::new();
        let mut changed = false;
        if self.clock.automatic_ready(BOT_INTERVAL) {
            self.clock.restart_automatic_wait();
            let mut bots: Vec<_> = self
                .room
                .players
                .iter()
                .filter(|player| player.is_bot && !player.left)
                .map(|player| player.id)
                .collect();
            // Avoid giving the lowest seat the first chance at every shared bot tick.
            fastrand::shuffle(&mut bots);
            for player in bots {
                if self.play_bot_action(player) {
                    changed = true;
                    deliveries.extend(self.after_action());
                }
                if self.game.as_ref().expect("active race").finished() {
                    break;
                }
            }
        }
        if changed || self.guess_seconds_remaining() != before {
            if !changed {
                self.bump_revision();
            }
            deliveries.extend(self.broadcast_game(None));
        }
        deliveries
    }
}
