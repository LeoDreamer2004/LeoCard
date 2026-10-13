use super::bot::Policy as BotPolicy;
use leocard_qigui523::{GameState, QiGuiCard, QiGuiPlayerId};

pub(super) struct Policy;

impl Policy {
    pub fn choose(game: &GameState, player: QiGuiPlayerId) -> Option<Vec<QiGuiCard>> {
        BotPolicy::choose(game, player)
    }
}
