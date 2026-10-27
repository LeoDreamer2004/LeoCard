use super::BlackjackSession;
use crate::new_match_id;
use leocard_blackjack::{BlackjackCard, GameState, build_shoe};
use std::time::Duration;

impl BlackjackSession {
    pub(super) fn begin_match(&mut self) {
        self.room.remove_departed_players();
        self.order = self.room.players.iter().map(|player| player.id).collect();
        self.match_id = Some(new_match_id());
        self.finished_reference_changes = None;
        self.game = Some(
            GameState::new(
                self.rules,
                self.order.len() as u8,
                self.initial_shoe.take().unwrap_or_else(shuffled_shoe),
            )
            .expect("validated rules and shoe"),
        );
        self.reset_round();
    }

    pub(super) fn continue_match(&mut self) {
        if self.game.as_ref().expect("active match").match_complete() {
            self.begin_match();
        } else {
            self.game
                .as_mut()
                .expect("active match")
                .start_next_round(shuffled_shoe())
                .expect("complete round and valid shoe");
            self.reset_round();
        }
    }

    fn reset_round(&mut self) {
        self.elapsed = Duration::ZERO;
        for player in &mut self.room.players {
            player.ready = false;
        }
        self.room.bump_revision();
    }

    pub(super) fn everyone_ready(&self) -> bool {
        let active: Vec<_> = self
            .room
            .players
            .iter()
            .filter(|player| !player.left && (player.connected || player.is_bot))
            .collect();
        !active.is_empty() && active.iter().all(|player| player.ready)
    }
}

fn shuffled_shoe() -> Vec<BlackjackCard> {
    let mut shoe = build_shoe();
    fastrand::shuffle(&mut shoe);
    shoe
}
