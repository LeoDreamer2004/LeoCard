use crate::app::games::texas::TEXAS_RUNOUT_REVEAL_DURATION;
use crate::app::presentation::GameSummaryAnimation;
use bevy::prelude::Component;
use leocard_protocol::TexasSnapshot;

/// One clock drives both winner reveals, the intervening board change and the final payout.
#[derive(Clone, Copy, Debug)]
pub(crate) struct TexasShowdownSequence {
    initial_delay: f32,
    twice: bool,
}

impl TexasShowdownSequence {
    pub(crate) const REVEAL_DURATION: f32 = 2.6;
    pub(crate) const DROP_START: f32 = Self::REVEAL_DURATION + 0.20;
    pub(crate) const DROP_DURATION: f32 = 0.50;
    pub(crate) const DEAL_START: f32 = Self::DROP_START + Self::DROP_DURATION;
    pub(crate) const DEAL_INTERVAL: f32 = 0.065;
    pub(crate) const DEAL_DURATION: f32 = 0.25;
    pub(crate) const FLIP_START: f32 =
        Self::DEAL_START + 4.0 * Self::DEAL_INTERVAL + Self::DEAL_DURATION + 0.10;
    pub(crate) const SECOND_REVEAL_START: f32 = Self::FLIP_START + TEXAS_RUNOUT_REVEAL_DURATION;
    pub(crate) const PAYOUT_START: f32 = Self::SECOND_REVEAL_START + Self::REVEAL_DURATION;

    pub(crate) fn new(game: &TexasSnapshot, initial_delay: f32) -> Self {
        Self {
            initial_delay,
            twice: !game.second_community.is_empty(),
        }
    }

    pub(crate) fn duration(self) -> f32 {
        self.initial_delay
            + if self.twice {
                Self::PAYOUT_START + Self::REVEAL_DURATION
            } else {
                Self::REVEAL_DURATION
            }
    }

    pub(crate) fn elapsed(self, animation: &GameSummaryAnimation) -> f32 {
        animation.elapsed + self.duration()
    }

    pub(crate) fn board_elapsed(self, animation: &GameSummaryAnimation) -> f32 {
        self.elapsed(animation) - self.initial_delay
    }

    pub(crate) fn payout_delay(self) -> f32 {
        self.initial_delay + if self.twice { Self::PAYOUT_START } else { 0.0 }
    }

    pub(super) fn round(self, run: u8) -> TexasShowdownRound {
        TexasShowdownRound {
            sequence: self,
            start: self.initial_delay
                + if run == 2 {
                    Self::SECOND_REVEAL_START
                } else {
                    0.0
                },
        }
    }
}

#[derive(Component, Clone, Copy)]
pub(in crate::app::games::texas) struct TexasShowdownRound {
    sequence: TexasShowdownSequence,
    start: f32,
}

impl TexasShowdownRound {
    pub(super) fn elapsed(self, animation: &GameSummaryAnimation) -> f32 {
        self.sequence.elapsed(animation) - self.start
    }

    pub(super) fn active(self, animation: &GameSummaryAnimation) -> bool {
        animation.match_id.is_some()
            && (0.0..TexasShowdownSequence::REVEAL_DURATION).contains(&self.elapsed(animation))
    }
}
