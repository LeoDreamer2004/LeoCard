use leocard_texas_holdem::{GameState, PassiveAutoPlay, TexasHoldemAction, TexasHoldemPlayerId};

pub(super) struct Policy;

impl Policy {
    pub fn choose(game: &GameState, player: TexasHoldemPlayerId) -> Option<TexasHoldemAction> {
        PassiveAutoPlay::choose_for_game(game, player)
    }
}
