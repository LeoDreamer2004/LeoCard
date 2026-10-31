use super::DoudizhuSession;
use super::statistics::DoudizhuStatistics;
use crate::new_match_id;
use leocard_doudizhu::achievements::DoudizhuAchievementTracker;
use leocard_doudizhu::{
    DoudizhuCard, DoudizhuGameState, DoudizhuMatchState, DoudizhuProfileStats, build_deck,
};

use leocard_game_common::PlayerId;
use leocard_protocol::DoudizhuEvent;

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

    pub(super) fn finish_hand(&mut self) -> Vec<DoudizhuEvent> {
        let game = self.game.as_ref().expect("active game");
        let result = game.result.as_ref().expect("finished hand");
        let state = self.match_state.as_mut().expect("active match");
        let before = state.scores().to_vec();
        assert!(state.finish_hand(result));
        let achievements =
            self.achievements
                .finish_hand(game, &before, state.scores(), state.complete());
        self.statistics.finish_hand(
            game.landlord.expect("finished hand has a landlord"),
            &result.scores,
        );
        if let Some(deltas) = state.reference_deltas(self.rules.mode) {
            self.room.settle_completed_match_profiles_once(
                &mut self.finished_reference_changes,
                deltas
                    .into_iter()
                    .enumerate()
                    .map(|(player, delta)| (PlayerId(player as u8), delta)),
                |player, delta| {
                    let stats = player
                        .game_profiles
                        .doudizhu
                        .get_or_insert_with(DoudizhuProfileStats::default);
                    self.statistics.merge_into(player.id, stats, delta);
                },
            );
        }
        self.room.prepare_rematch();
        achievements
            .into_iter()
            .map(|(player, statistics)| DoudizhuEvent::ActionAnalyzed {
                player,
                statistics: Box::new(statistics),
            })
            .collect()
    }
}
