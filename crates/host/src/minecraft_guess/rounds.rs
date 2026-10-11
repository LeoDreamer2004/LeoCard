use super::MinecraftGuessSession;
use crate::new_match_id;
use leocard_minecraft_guess::{Catalogue, GameState, MatchState};
use std::time::Duration;

impl MinecraftGuessSession {
    pub(super) fn begin_match(&mut self) {
        self.room.remove_departed_players();
        self.match_state = Some(MatchState::new(
            self.room.players.iter().map(|player| player.id.0).collect(),
            self.rules.rounds,
        ));
        self.match_id = Some(new_match_id());
        self.finished_reference_changes = None;
        self.begin_round();
    }

    pub(super) fn continue_match(&mut self) {
        let advanced = self.match_state.as_mut().expect("active match").advance();
        if advanced {
            self.begin_round();
        } else {
            self.begin_match();
        }
    }

    fn begin_round(&mut self) {
        let entries: Vec<_> = Catalogue::bundled().entries(self.rules.pool).collect();
        let answer = entries[fastrand::usize(..entries.len())].clone();
        let players = self
            .match_state
            .as_ref()
            .expect("active match")
            .turn_order();
        self.game = Some(GameState::new(answer, players, self.rules));
        self.automatic_wait = Duration::ZERO;
        for player in &mut self.room.players {
            player.ready = false;
        }
        self.bump_revision();
    }
}
