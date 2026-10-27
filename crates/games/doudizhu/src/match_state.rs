use crate::{GameError, Mode, ResultView, RuleSet};
use leocard_game_common::ranked_awards;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MatchProgress {
    pub hand_number: u8,
    pub rounds: u8,
    pub player_count: u8,
}

impl MatchProgress {
    pub const fn round(self) -> u8 {
        (self.hand_number - 1) / self.player_count + 1
    }

    pub const fn hand_in_round(self) -> u8 {
        (self.hand_number - 1) % self.player_count + 1
    }

    pub const fn total_hands(self) -> u8 {
        self.rounds * self.player_count
    }

    pub const fn is_last_hand(self) -> bool {
        self.hand_number == self.total_hands()
    }
}

#[derive(Clone, Debug)]
pub struct MatchState {
    progress: MatchProgress,
    first: u8,
    scores: Vec<i32>,
    hand_finished: bool,
}

impl MatchState {
    pub fn new(rules: RuleSet, first: u8) -> Result<Self, GameError> {
        if !rules.is_valid() {
            return Err(GameError::InvalidRules);
        }
        if first >= rules.mode.players() {
            return Err(GameError::InvalidPlayer);
        }
        Ok(Self {
            progress: MatchProgress {
                hand_number: 1,
                rounds: rules.rounds,
                player_count: rules.mode.players(),
            },
            first,
            scores: vec![0; usize::from(rules.mode.players())],
            hand_finished: false,
        })
    }

    pub const fn progress(&self) -> MatchProgress {
        self.progress
    }

    pub const fn first_bidder(&self) -> u8 {
        (self.first + self.progress.hand_number - 1) % self.progress.player_count
    }

    pub fn scores(&self) -> &[i32] {
        &self.scores
    }

    pub const fn complete(&self) -> bool {
        self.hand_finished && self.progress.is_last_hand()
    }

    pub fn finish_hand(&mut self, result: &ResultView) -> bool {
        if self.hand_finished || result.scores.len() != self.scores.len() {
            return false;
        }
        for (total, score) in self.scores.iter_mut().zip(&result.scores) {
            *total = total.saturating_add(*score);
        }
        self.hand_finished = true;
        true
    }

    pub fn next_hand(&mut self) -> bool {
        if !self.hand_finished || self.progress.is_last_hand() {
            return false;
        }
        self.progress.hand_number += 1;
        self.hand_finished = false;
        true
    }

    pub fn reference_deltas(&self, mode: Mode) -> Option<Vec<i16>> {
        if !self.complete() {
            return None;
        }
        let awards: &[i16] = match (mode, self.progress.rounds >= 3) {
            (Mode::Three, false) => &[3, 0, -3],
            (Mode::Four, false) => &[4, 2, -2, -4],
            (Mode::Three, true) => &[5, 0, -5],
            (Mode::Four, true) => &[6, 3, -3, -6],
        };
        ranked_awards(&self.scores, awards)
    }
}
