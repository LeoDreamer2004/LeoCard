use crate::automation::BotDecision;
use leocard_game_common::PlayerId;

use leocard_texas_holdem::{TexasAction, TexasBot, TexasBotObservation, TexasGameState};

pub(super) struct Policy;

impl Policy {
    pub fn choose(
        game: &TexasGameState,
        player: PlayerId,
        decision: &mut BotDecision<TexasBotObservation, TexasAction>,
    ) -> Option<TexasAction> {
        let observation = TexasBotObservation::from_game(game, player)?;
        if let Some(action) = observation.immediate_action() {
            decision.clear();
            return Some(action);
        }
        let fallback = observation.fallback();
        decision.poll(observation, fallback, TexasBot::choose)
    }
}
