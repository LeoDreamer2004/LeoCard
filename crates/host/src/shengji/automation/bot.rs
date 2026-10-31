use crate::automation::BotDecision;
use leocard_game_common::PlayerId;

use leocard_shengji::{
    ShengjiActionOutcome, ShengjiBot, ShengjiBotAction, ShengjiBotObservation, ShengjiGameState,
};

pub(super) struct Policy;

impl Policy {
    pub fn act(
        game: &mut ShengjiGameState,
        player: PlayerId,
        decision: &mut BotDecision<ShengjiBotObservation, ShengjiBotAction>,
    ) -> Option<ShengjiActionOutcome> {
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
