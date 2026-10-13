use leocard_minecraft_guess::GameState;

pub(super) struct Policy;

impl Policy {
    pub fn act(game: &mut GameState, current: u8) {
        game.skip(current).expect("current player can skip");
    }
}
