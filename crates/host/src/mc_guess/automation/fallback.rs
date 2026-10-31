use leocard_game_common::PlayerId;
use leocard_mc_guess::McGuessGameState;

pub(super) struct Policy;

impl Policy {
    pub fn act(game: &mut McGuessGameState, current: PlayerId) {
        game.skip(current).expect("current player can skip");
    }
}
