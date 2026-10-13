use super::action::AutomaticMahjongAction;
use crate::automation::BotDecision;
use leocard_mahjong::{GameState, MahjongBot, MahjongBotObservation, MahjongPlayerId};

pub(super) struct Policy;

impl Policy {
    pub fn choose(
        game: &GameState,
        player: MahjongPlayerId,
        decision: &mut BotDecision<MahjongBotObservation, AutomaticMahjongAction>,
    ) -> Option<AutomaticMahjongAction> {
        let observation = MahjongBotObservation::from_game(game, player)?;
        if let Some(action) = observation.immediate_action() {
            decision.clear();
            return Some(action);
        }
        let fallback = observation.fallback();
        decision.poll(observation, fallback, MahjongBot::choose)
    }
}
