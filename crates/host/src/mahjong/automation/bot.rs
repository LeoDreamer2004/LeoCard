use crate::automation::BotDecision;
use leocard_game_common::PlayerId;
use leocard_mahjong::{MahjongBot, MahjongBotAction, MahjongBotObservation, MahjongGameState};

pub(super) struct Policy;

impl Policy {
    pub fn choose(
        game: &MahjongGameState,
        player: PlayerId,
        decision: &mut BotDecision<MahjongBotObservation, MahjongBotAction>,
    ) -> Option<MahjongBotAction> {
        let observation = MahjongBotObservation::from_game(game, player)?;
        if let Some(action) = observation.immediate_action() {
            decision.clear();
            return Some(action);
        }
        let fallback = observation.fallback();
        decision.poll(observation, fallback, MahjongBot::choose)
    }
}
