use crate::automation::BotDecision;
use leocard_shengji::{
    ActionOutcome, GameState, ShengjiBot, ShengjiBotAction, ShengjiBotObservation, ShengjiPlayerId,
};

pub(super) struct Policy;

impl Policy {
    pub fn act(
        game: &mut GameState,
        player: ShengjiPlayerId,
        decision: &mut BotDecision<ShengjiBotObservation, ShengjiBotAction>,
    ) -> Option<ActionOutcome> {
        let observation = ShengjiBotObservation::from_game(game, player)?;
        let fallback = observation.fallback();
        let action = decision.poll(observation, fallback, ShengjiBot::choose)?;
        match action {
            ShengjiBotAction::Play(cards) => game.play_cards(player, &cards),
            ShengjiBotAction::Bury(cards) => game.bury(player, &cards),
            ShengjiBotAction::Copy(cards) => game.choose_bottom_copy(player, cards.as_deref()),
            ShengjiBotAction::Cross(cards) => {
                game.choose_five_trump_crossing(player, cards.as_deref())
            }
            ShengjiBotAction::Return(cards) => game.return_five_trump_crossing(player, &cards),
        }
        .ok()
    }
}
