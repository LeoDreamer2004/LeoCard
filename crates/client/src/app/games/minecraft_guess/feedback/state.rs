use bevy::prelude::*;
use leocard_protocol::MatchId;
use std::collections::VecDeque;

pub(super) const ROW_DURATION: f32 = 0.64;
pub(super) const ANSWER_DURATION: f32 = 1.55;

#[derive(Clone, Copy)]
pub(super) enum FeedbackSound {
    Submit,
    Reveal,
    Success,
    Skip,
    Turn,
}

pub(super) struct SoundCue {
    pub at: f32,
    pub sound: FeedbackSound,
}

pub(super) struct RowReveal {
    pub start: f32,
    pub columns: usize,
}

/// Playback belongs to the small game, rather than the entities rebuilt for its UI.
#[derive(Resource, Default)]
pub(crate) struct GuessFeedback {
    pub(super) key: Option<(MatchId, u8)>,
    pub(super) clock: f32,
    pub(super) rows: Vec<Option<RowReveal>>,
    pub(super) cues: VecDeque<SoundCue>,
    pub(super) finished_at: Option<f32>,
    pub(super) last_player: Option<u8>,
    pub(super) skipped: Vec<u8>,
}

impl GuessFeedback {
    pub(super) fn tail(&self) -> f32 {
        self.rows
            .iter()
            .flatten()
            .last()
            .map_or(self.clock, |row| row.start + ROW_DURATION)
            .max(self.clock)
    }

    pub(super) fn cue(&mut self, at: f32, sound: FeedbackSound) {
        let index = self.cues.partition_point(|cue| cue.at <= at);
        self.cues.insert(index, SoundCue { at, sound });
    }

    pub(super) fn cell_progress(&self, row: usize, column: usize) -> f32 {
        let Some(Some(reveal)) = self.rows.get(row) else {
            return 1.0;
        };
        let delay = if column == 0 {
            0.0
        } else {
            0.10 + (column - 1) as f32 * 0.37 / reveal.columns.saturating_sub(2).max(1) as f32
        };
        ((self.clock - reveal.start - delay) / 0.16).clamp(0.0, 1.0)
    }

    pub(super) fn answer_elapsed(&self) -> Option<f32> {
        self.finished_at.map(|start| self.clock - start)
    }

    pub(super) fn answer_progress(&self) -> f32 {
        self.answer_elapsed()
            .map_or(0.0, |elapsed| (elapsed / 0.24).clamp(0.0, 1.0))
    }

    pub(super) fn banner_opacity(&self) -> f32 {
        self.answer_elapsed().map_or(0.0, |elapsed| {
            (elapsed / 0.20).clamp(0.0, 1.0) * ((ANSWER_DURATION - elapsed) / 0.30).clamp(0.0, 1.0)
        })
    }

    pub(crate) fn scoring_ready(&self) -> bool {
        self.tail() <= self.clock && self.answer_elapsed().is_none_or(|elapsed| elapsed >= 0.48)
    }

    pub(crate) fn busy(&self) -> bool {
        self.tail() > self.clock
            || self
                .answer_elapsed()
                .is_some_and(|elapsed| elapsed < ANSWER_DURATION)
    }
}
