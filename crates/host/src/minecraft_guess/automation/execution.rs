use super::super::MinecraftGuessSession;
use super::fallback;
use crate::Delivery;
use crate::lifecycle::HostedGameLifecycle;
use fastrand::Rng;
use leocard_minecraft_guess::{
    GuessMode, MinecraftGuessBot, MinecraftGuessBotAction, MinecraftGuessBotObservation,
};
use std::time::Duration;

const AUTOMATIC_ACTION_DELAY: Duration = Duration::from_secs(2);

impl MinecraftGuessSession {
    pub(in super::super) fn guess_seconds_remaining(&self) -> Option<u16> {
        if self.generation.is_some() {
            return None;
        }
        let game = self.game.as_ref()?;
        if game.finished() {
            return None;
        }
        self.clock.remaining(self.rules.guess_seconds())
    }

    pub fn advance_time(&mut self, elapsed: Duration) -> Vec<Delivery> {
        if self.generation.is_some() {
            return self.advance_generation();
        }
        if !self.game.as_ref().is_some_and(|game| !game.finished()) {
            return Vec::new();
        }
        self.game
            .as_mut()
            .expect("active game")
            .advance_selection_time(elapsed);
        match self.rules.mode {
            GuessMode::TurnBased | GuessMode::Graph => self.advance_turn(elapsed),
            GuessMode::Race => self.advance_race(elapsed),
        }
    }

    pub(super) fn play_bot_action(&mut self, player: u8) -> bool {
        let game = self.game.as_mut().expect("an automatic action has a game");
        let Some(observation) = MinecraftGuessBotObservation::from_game(game, player) else {
            return false;
        };
        let Some(action) = MinecraftGuessBot::choose(&observation, &mut Rng::new()) else {
            return false;
        };
        if let Some(node) = action.target_node()
            && game.select_node(player, node).is_err()
        {
            return false;
        }
        match action {
            MinecraftGuessBotAction::Guess { entry_id, .. } => game.guess(player, &entry_id),
            MinecraftGuessBotAction::Skip { .. } => game.skip(player),
        }
        .expect("a public bot candidate is a legal action");
        true
    }

    fn advance_turn(&mut self, elapsed: Duration) -> Vec<Delivery> {
        let Some(current) = self.game.as_ref().and_then(|game| game.current_player()) else {
            return Vec::new();
        };
        let before = self.guess_seconds_remaining();
        let player = &self.room.players[usize::from(current)];
        let automatic = !player.connected || player.is_bot || player.left;
        self.clock.advance(elapsed, automatic);
        let expired = self.guess_seconds_remaining() == Some(0);
        if expired || (automatic && self.clock.automatic_ready(AUTOMATIC_ACTION_DELAY)) {
            if player.is_bot && !player.left {
                self.play_bot_action(current);
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
