use super::BlackjackSession;
use crate::Delivery;
use crate::lifecycle::HostedGameLifecycle;
use leocard_blackjack::{BlackjackAutoPlay, BlackjackBot, BlackjackObservation, Phase};
use std::time::Duration;

impl BlackjackSession {
    pub(super) fn seconds_remaining(&self) -> Option<u16> {
        let game = self.game.as_ref()?;
        if game.phase() == Phase::Complete || self.rules.turn_seconds == 0 {
            return None;
        }
        Some(
            self.rules
                .turn_seconds
                .saturating_sub(self.elapsed.as_secs().min(u64::from(u16::MAX)) as u16),
        )
    }

    pub fn advance_time(&mut self, elapsed: Duration) -> Vec<Delivery> {
        if elapsed.is_zero() || self.room.closed {
            return Vec::new();
        }
        let Some(game) = self.game.as_ref() else {
            return Vec::new();
        };
        if game.phase() == Phase::Complete {
            if !self.everyone_ready() {
                self.elapsed = Duration::ZERO;
                return Vec::new();
            }
            self.elapsed += elapsed;
            let dealer_draws = game.visible_dealer().len().saturating_sub(2) as u64;
            let result_hold = Duration::from_secs(4 + dealer_draws);
            if self.elapsed < result_hold {
                return Vec::new();
            }
            self.continue_match();
            return self.broadcast_game(None);
        }
        let index = game.current_player().expect("live round has a turn");
        let player = self.order[index];
        let participant = self
            .room
            .players
            .iter()
            .find(|participant| participant.id == player)
            .expect("match participant");
        let automatic = participant.is_bot
            || participant.auto_play
            || !participant.connected
            || participant.left;
        let is_bot = participant.is_bot;
        let action_interval = if is_bot {
            Duration::from_secs(2)
        } else {
            Duration::from_millis(1100)
        };
        let before = self.seconds_remaining();
        self.elapsed += elapsed;
        let expired = self.rules.turn_seconds > 0
            && self.elapsed >= Duration::from_secs(u64::from(self.rules.turn_seconds));
        if (automatic && self.elapsed >= action_interval) || expired {
            let game = self.game.as_ref().expect("active game");
            let observation =
                BlackjackObservation::for_player(game, index).expect("current turn observation");
            let action = if !automatic {
                BlackjackAutoPlay::timeout(game.phase(), self.rules.minimum_bet)
            } else if is_bot {
                BlackjackBot::choose(&observation)
            } else {
                BlackjackAutoPlay::choose(&observation)
            };
            let mut deliveries = self
                .apply_action(player, action)
                .expect("automatic strategy returns a legal action");
            deliveries.extend(self.broadcast_game(None));
            return deliveries;
        }
        if self.seconds_remaining() != before {
            self.room.bump_revision();
            return self.broadcast_game(None);
        }
        Vec::new()
    }
}
