use super::generation::GraphGeneration;
use super::{GuessClock, McGuessSession};
use crate::new_match_id;
use leocard_game_common::PlayerId;
use leocard_mc_guess::{
    McGuessCatalogue, McGuessGameError, McGuessGameState, McGuessMatchState, McGuessMode,
};

impl McGuessSession {
    pub(super) fn begin_match(&mut self) -> Result<(), McGuessGameError> {
        let mut room = self.room.clone();
        room.remove_departed_players();
        let mut players: Vec<_> = room.players.iter().map(|player| player.id).collect();
        if self.rules.mode == McGuessMode::Graph && !self.rules.graph.round_robin {
            fastrand::shuffle(&mut players);
        }
        let state = McGuessMatchState::new(players, self.rules);
        let game = self.new_round(state.turn_order())?;
        self.process_presence.clear();
        self.room = room;
        self.match_state = Some(state);
        self.match_id = Some(new_match_id());
        self.finished_reference_changes = None;
        self.install_round(game);
        Ok(())
    }

    pub(super) fn continue_match(&mut self) -> Result<(), McGuessGameError> {
        let mut state = self.match_state.clone().expect("active match");
        if state.advance() {
            let game = self.new_round(state.turn_order())?;
            self.match_state = Some(state);
            self.install_round(game);
            Ok(())
        } else {
            self.begin_match()
        }
    }

    fn new_round(&mut self, players: Vec<PlayerId>) -> Result<McGuessGameState, McGuessGameError> {
        let rules = self.rules;
        if rules.mode == McGuessMode::Graph {
            self.generation = Some(GraphGeneration::new(players.clone(), rules));
            return Ok(McGuessGameState::pending_graph(players, rules));
        }
        let entries: Vec<_> = McGuessCatalogue::bundled()
            .entries(rules.options().pool)
            .collect();
        let answer = entries[fastrand::usize(..entries.len())].clone();
        Ok(McGuessGameState::new(answer, players, rules))
    }

    fn install_round(&mut self, game: McGuessGameState) {
        self.game = Some(game);
        self.clock = GuessClock::default();
        for player in &mut self.room.players {
            player.ready = false;
        }
        self.bump_revision();
    }
}
