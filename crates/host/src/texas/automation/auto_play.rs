use leocard_game_common::PlayerId;
use leocard_texas_holdem::{TexasAction, TexasGameState, TexasPassiveAutoPlay};

pub(super) struct Policy;

impl Policy {
    pub fn choose(game: &TexasGameState, player: PlayerId) -> Option<TexasAction> {
        TexasPassiveAutoPlay::choose_for_game(game, player)
    }
}
