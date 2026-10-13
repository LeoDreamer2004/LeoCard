use crate::automation::BotDecision;
use leocard_texas_holdem::{
    GameState, TexasHoldemAction, TexasHoldemBot, TexasHoldemBotObservation, TexasHoldemPlayerId,
};

pub(super) struct Policy;

impl Policy {
    pub fn choose(
        game: &GameState,
        player: TexasHoldemPlayerId,
        decision: &mut BotDecision<TexasHoldemBotObservation, TexasHoldemAction>,
    ) -> Option<TexasHoldemAction> {
        let observation = TexasHoldemBotObservation::from_game(game, player)?;
        if let Some(action) = observation.immediate_action() {
            decision.clear();
            return Some(action);
        }
        let fallback = observation.fallback();
        decision.poll(observation, fallback, TexasHoldemBot::choose)
    }
}
