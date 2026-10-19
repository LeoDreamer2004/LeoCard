use super::{GuessClock, MinecraftGuessSession};
use crate::new_match_id;
use leocard_minecraft_guess::{
    Catalogue, GameError, GameState, GuessMode, MatchState, MinecraftGuessRuleSet,
};

impl MinecraftGuessSession {
    pub(super) fn begin_match(&mut self) -> Result<(), GameError> {
        let mut room = self.room.clone();
        room.remove_departed_players();
        let players = room.players.iter().map(|player| player.id.0).collect();
        let state = MatchState::new(players, self.rules);
        let game = Self::new_round(state.turn_order(), self.rules)?;
        self.process_presence.clear();
        self.room = room;
        self.match_state = Some(state);
        self.match_id = Some(new_match_id());
        self.finished_reference_changes = None;
        self.install_round(game);
        Ok(())
    }

    pub(super) fn continue_match(&mut self) -> Result<(), GameError> {
        let mut state = self.match_state.clone().expect("active match");
        if state.advance() {
            let game = Self::new_round(state.turn_order(), self.rules)?;
            self.match_state = Some(state);
            self.install_round(game);
            Ok(())
        } else {
            self.begin_match()
        }
    }

    fn new_round(players: Vec<u8>, rules: MinecraftGuessRuleSet) -> Result<GameState, GameError> {
        if rules.mode == GuessMode::Graph {
            return GameState::new_graph(players, rules, fastrand::u64(..));
        }
        let entries: Vec<_> = Catalogue::bundled().entries(rules.options().pool).collect();
        let answer = entries[fastrand::usize(..entries.len())].clone();
        Ok(GameState::new(answer, players, rules))
    }

    fn install_round(&mut self, game: GameState) {
        self.game = Some(game);
        self.clock = GuessClock::default();
        for player in &mut self.room.players {
            player.ready = false;
        }
        self.bump_revision();
    }
}
