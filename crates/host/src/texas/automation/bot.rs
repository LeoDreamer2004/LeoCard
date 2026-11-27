use super::super::TexasSession;
use crate::automation::BotDecision;
use leocard_ai::LoadedModel;
use leocard_game_common::PlayerId;
use leocard_texas_holdem::{
    TexasAction, TexasBot, TexasBotObservation, TexasGameState, TexasPpoBot, TexasPpoMemory,
    TexasPpoObservation,
};

impl TexasSession {
    pub(in super::super) fn refresh_bot_model(&mut self) {
        if self.room.bot_ai.is_some()
            && self
                .game
                .as_ref()
                .is_some_and(|game| !TexasPpoBot::supports_game(game.game()))
        {
            self.room.disable_bot_ai();
            self.bot_decision.clear();
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(in super::super) struct BotObservation {
    ordinary: TexasBotObservation,
    neural: Option<TexasPpoObservation>,
}

pub(super) struct Policy;

impl Policy {
    pub fn choose(
        game: &TexasGameState,
        player: PlayerId,
        decision: &mut BotDecision<BotObservation, TexasAction>,
        model: Option<LoadedModel>,
        memory: &TexasPpoMemory,
    ) -> Option<TexasAction> {
        let observation = TexasBotObservation::from_game(game, player)?;
        if let Some(action) = observation.immediate_action() {
            decision.clear();
            return Some(action);
        }
        let fallback = observation.fallback();
        let neural = model
            .as_ref()
            .and_then(|_| TexasPpoObservation::from_game(game, player, memory));
        decision.poll(
            BotObservation {
                ordinary: observation,
                neural,
            },
            fallback,
            move |input, budget| {
                if let Some((model, observation)) = model.as_ref().zip(input.neural.as_ref())
                    && let Ok(action) = TexasPpoBot::choose(observation, model)
                {
                    return action;
                }
                TexasBot::choose(&input.ordinary, budget)
            },
        )
    }
}
