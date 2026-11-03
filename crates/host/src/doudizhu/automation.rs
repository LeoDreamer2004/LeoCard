use super::DoudizhuSession;
use leocard_doudizhu::{
    DoudizhuAction, DoudizhuGameState, DoudizhuMode, DouzeroBot, DouzeroObservation, FourPlayerBot,
    FourPlayerObservation, auto_play_action, bot_action,
};
use leocard_game_common::{PlayerId, SearchBudget};

#[derive(Clone, Debug, PartialEq)]
pub(super) enum DoudizhuBotObservation {
    Three(DouzeroObservation),
    Four(FourPlayerObservation),
}

impl DoudizhuBotObservation {
    fn from_game(game: &DoudizhuGameState, history: &[(PlayerId, DoudizhuAction)]) -> Option<Self> {
        match game.rules.mode {
            DoudizhuMode::Three => DouzeroObservation::from_game(game, history).map(Self::Three),
            DoudizhuMode::Four => FourPlayerObservation::from_game(game).map(Self::Four),
        }
    }

    fn choose(
        &self,
        budget: &SearchBudget,
        report: &mut dyn FnMut(DoudizhuAction),
    ) -> DoudizhuAction {
        match self {
            Self::Three(input) => DouzeroBot::choose_with_progress(input, budget, report),
            Self::Four(input) => FourPlayerBot::choose_with_progress(input, budget, report),
        }
    }
}

impl DoudizhuSession {
    pub(super) fn choose_automatic_action(&mut self) -> Option<DoudizhuAction> {
        let game = self.game.as_ref()?;
        let participant = &self.room.players[usize::from(game.current)];
        // Human auto-play, disconnection and departure deliberately retain the old policy.
        if !participant.is_bot {
            self.bot_decision.clear();
            return auto_play_action(game);
        }
        let fallback = bot_action(game)?;
        let Some(observation) = DoudizhuBotObservation::from_game(game, &self.bot_history) else {
            return Some(fallback); // Bidding retains the existing hand-only policy.
        };
        self.bot_decision.poll_with_progress(
            observation,
            |input| match input {
                DoudizhuBotObservation::Four(input) => FourPlayerBot::fallback(input),
                DoudizhuBotObservation::Three(_) => fallback,
            },
            DoudizhuBotObservation::choose,
        )
    }
}
