use super::DoudizhuSession;
use crate::new_match_id;
use leocard_doudizhu::{Card, GameState, MatchState, build_deck};
use leocard_protocol::PlayerId;

impl DoudizhuSession {
    pub(super) fn begin_match(&mut self) {
        self.match_state = Some(
            MatchState::new(self.rules, fastrand::u8(..self.rules.mode.players()))
                .expect("validated rules"),
        );
        let mut deck = self
            .deck
            .take()
            .unwrap_or_else(|| build_deck(self.rules.mode));
        if self.match_id.is_some() {
            fastrand::shuffle(&mut deck);
        }
        self.match_id = Some(new_match_id());
        self.finished_reference_changes = None;
        for player in &mut self.room.players {
            player.auto_play = false;
        }
        self.deal_hand(deck);
    }

    pub(super) fn continue_match(&mut self) {
        if self.match_state.as_ref().expect("active match").complete() {
            self.begin_match();
        } else {
            assert!(self.match_state.as_mut().expect("active match").next_hand());
            self.redeal_hand();
        }
    }

    pub(super) fn redeal_hand(&mut self) {
        let mut deck = build_deck(self.rules.mode);
        fastrand::shuffle(&mut deck);
        self.deal_hand(deck);
    }

    fn deal_hand(&mut self, deck: Vec<Card>) {
        let first = self
            .match_state
            .as_ref()
            .expect("active match")
            .first_bidder();
        self.game =
            Some(GameState::new(self.rules, deck, first).expect("validated deck and rules"));
        for player in &mut self.room.players {
            player.ready = player.is_bot;
        }
        self.reset_automatic_delay();
    }

    pub(super) fn everyone_ready(&self) -> bool {
        self.room
            .players
            .iter()
            .all(|player| player.ready || player.is_bot || player.left || !player.connected)
    }

    pub(super) fn finish_hand(&mut self) {
        let result = self
            .game
            .as_ref()
            .expect("active game")
            .result
            .as_ref()
            .expect("finished hand");
        let state = self.match_state.as_mut().expect("active match");
        assert!(state.finish_hand(result));
        if let Some(deltas) = state.reference_deltas(self.rules.mode) {
            self.room.settle_completed_match_profiles_once(
                &mut self.finished_reference_changes,
                deltas
                    .into_iter()
                    .enumerate()
                    .map(|(player, delta)| (PlayerId(player as u8), delta)),
                |_, _| {},
            );
        }
        self.room.prepare_rematch();
    }
}
