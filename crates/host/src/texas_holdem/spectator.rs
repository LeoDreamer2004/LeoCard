use leocard_texas_holdem::{GameState, Phase};

#[derive(Clone, Debug)]
pub(super) struct SpectatorAccess {
    folded_at: Vec<Option<usize>>,
    requested: Vec<bool>,
}

impl SpectatorAccess {
    pub fn new(players: usize) -> Self {
        Self {
            folded_at: vec![None; players],
            requested: vec![false; players],
        }
    }

    pub fn record_fold(&mut self, player: usize, board_len: usize) {
        self.folded_at[player] = Some(board_len);
    }

    pub fn start_hand(&mut self) {
        self.folded_at.fill(None);
    }

    pub fn available(&self, game: &GameState, player: usize) -> bool {
        matches!(game.phase(), Phase::Betting(_))
            && game.players()[player].folded()
            && self.folded_at[player].is_some_and(|board_len| game.community().len() > board_len)
            && game
                .players()
                .iter()
                .filter(|player| !player.folded())
                .count()
                >= 2
    }

    pub fn requested(&self, player: usize) -> bool {
        self.requested[player]
    }

    pub fn set_requested(&mut self, player: usize, enabled: bool) {
        self.requested[player] = enabled;
    }
}
