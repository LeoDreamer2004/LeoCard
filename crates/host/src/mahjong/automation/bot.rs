use crate::automation::BotDecision;
use leocard_ai::LoadedModel;
use leocard_game_common::PlayerId;
use leocard_mahjong::{
    KdensBot, MahjongBot, MahjongBotAction, MahjongBotObservation, MahjongGameState,
    MahjongModelHistory, MahjongModelObservation,
};

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum MahjongBotInput {
    Rules(MahjongBotObservation),
    Model(MahjongModelObservation),
}

pub(super) struct Policy;

impl Policy {
    pub fn choose(
        game: &MahjongGameState,
        player: PlayerId,
        decision: &mut BotDecision<MahjongBotInput, MahjongBotAction>,
        model: Option<LoadedModel>,
        history: &mut MahjongModelHistory,
    ) -> Option<MahjongBotAction> {
        let observation = if model.is_some() {
            MahjongBotInput::Model(history.observation(game, player)?)
        } else {
            MahjongBotInput::Rules(MahjongBotObservation::from_game(game, player)?)
        };
        let position = match &observation {
            MahjongBotInput::Rules(position) => position,
            MahjongBotInput::Model(observation) => &observation.position,
        };
        if let Some(action) = position.immediate_action() {
            decision.clear();
            return Some(action);
        }
        let fallback = position.fallback();
        decision.poll_with_progress(
            observation,
            |_| fallback,
            move |input, budget, report| match input {
                MahjongBotInput::Rules(position) => MahjongBot::choose(position, budget),
                MahjongBotInput::Model(observation) => KdensBot::choose_with_progress(
                    observation,
                    model.as_ref().expect("model input"),
                    budget,
                    report,
                )
                .unwrap_or(fallback),
            },
        )
    }
}
