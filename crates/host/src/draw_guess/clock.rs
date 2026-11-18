use super::DrawGuessSession;
use super::automation::DrawGuessBots;
use crate::Delivery;
use crate::features::MatchSettlement;
use crate::lifecycle::HostedGameLifecycle;
use leocard_draw_guess::DrawGuessPhase;
use std::time::Duration;

impl DrawGuessSession {
    pub fn advance_time(&mut self, elapsed: Duration) -> Vec<Delivery> {
        if self.room.closed || elapsed.is_zero() {
            return Vec::new();
        }
        let Some(game) = &mut self.game else {
            return Vec::new();
        };
        if game.phase() == DrawGuessPhase::Finished {
            return Vec::new();
        }
        let before = (game.turn(), game.phase());
        for player in &self.room.players {
            game.set_connected(
                player.id,
                (player.connected || player.is_bot) && !player.left,
            );
        }
        self.clock_remainder += elapsed;
        let millis = self.clock_remainder.as_millis().min(u128::from(u32::MAX)) as u32;
        self.clock_remainder -= Duration::from_millis(u64::from(millis));
        let clock_changed = DrawGuessBots::new(&self.room).advance(game, millis);
        let changed = clock_changed || before != (game.turn(), game.phase());
        if game.phase() == DrawGuessPhase::Finished {
            self.commit_match(&());
            self.room.prepare_rematch();
        }
        self.broadcast_elapsed += elapsed;
        if changed || self.broadcast_elapsed >= Duration::from_secs(1) {
            self.broadcast_elapsed = Duration::ZERO;
            self.room.bump_revision();
            self.broadcast_game(None)
        } else {
            Vec::new()
        }
    }
}
