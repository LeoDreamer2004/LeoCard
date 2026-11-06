use super::DoudizhuSession;
use super::statistics::DoudizhuStatistics;
use crate::new_match_id;
use leocard_doudizhu::achievements::DoudizhuAchievementTracker;
use leocard_doudizhu::{DoudizhuCard, DoudizhuGameState, DoudizhuMatchState, build_deck};
use leocard_game_common::PlayerId;

impl DoudizhuSession {
    pub(super) fn begin_match(&mut self) {
        self.statistics = DoudizhuStatistics::new(self.rules.mode.players());
        self.achievements = DoudizhuAchievementTracker::new(self.rules.mode.players());
        self.match_state = Some(
            DoudizhuMatchState::new(
                self.rules,
                PlayerId(fastrand::u8(..self.rules.mode.players())),
            )
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

    fn deal_hand(&mut self, deck: Vec<DoudizhuCard>) {
        self.bot_history.clear();
        self.achievements.begin_hand();
        let first = self
            .match_state
            .as_ref()
            .expect("active match")
            .first_bidder();
        self.game = Some(
            DoudizhuGameState::new(self.rules, deck, first).expect("validated deck and rules"),
        );
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
}
