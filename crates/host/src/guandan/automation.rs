use super::GuandanSession;
use leocard_game_common::PlayerId;
use leocard_guandan::{GuandanAction, GuandanActorBot, GuandanModelObservation, auto_play_action};

impl GuandanSession {
    pub(super) fn automatic_action(&mut self, player: PlayerId) -> Option<GuandanAction> {
        let game = self.game.as_ref()?;
        let fallback = auto_play_action(game, player)?;
        let model = self
            .room
            .bot_ai
            .as_ref()
            .filter(|_| self.room.players[player.index()].is_bot);
        let Some((model, observation)) = model
            .cloned()
            .zip(GuandanModelObservation::from_game(game, &self.bot_history))
        else {
            self.bot_decision.clear();
            return Some(fallback);
        };
        self.bot_decision
            .poll(observation, fallback.clone(), move |input, _| {
                GuandanActorBot::choose(input, &model).unwrap_or(fallback)
            })
    }
}
