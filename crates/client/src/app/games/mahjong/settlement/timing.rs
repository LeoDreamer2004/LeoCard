//! One settlement timeline shared by rendering and summary playback.

use super::super::mahjong_settlement_timing;
use leocard_mahjong::MahjongMatchLength;
use leocard_protocol::MahjongHandResultView;

pub(super) const PAGE_FADE_DURATION: f32 = 0.28;
pub(super) const SCORE_FADE_DURATION: f32 = 0.36;
pub(super) const DELTA_FLIGHT_DURATION: f32 = 0.62;
pub(super) const SCORE_ROLL_DURATION: f32 = 0.80;
pub(super) const DELTA_APPEAR_DELAY: f32 = 0.14;
pub(super) const DELTA_HOLD_DURATION: f32 = 0.75;
pub(super) const SCORE_END_HOLD_DURATION: f32 = 0.65;

#[derive(Clone, Copy)]
pub(super) struct ScoreTiming {
    pub(super) start: f32,
    pub(super) flight: f32,
    pub(super) roll: f32,
}

pub(super) struct SettlementTimeline {
    pub(super) pages_end: f32,
    pub(super) scores: Option<ScoreTiming>,
    pub(super) continue_at: f32,
}

impl SettlementTimeline {
    pub(super) fn new(result: &MahjongHandResultView) -> Self {
        let pages_end = mahjong_settlement_timing(result).score_rows_delay
            + if result.winners.is_empty() { 0.0 } else { 0.12 };
        let scores = (result.match_length != MahjongMatchLength::SingleHand).then(|| {
            let flight = pages_end + DELTA_APPEAR_DELAY + DELTA_HOLD_DURATION;
            ScoreTiming {
                start: pages_end,
                flight,
                roll: flight + DELTA_FLIGHT_DURATION * 0.72,
            }
        });
        Self {
            pages_end,
            scores,
            continue_at: scores.map_or(pages_end, |timing| {
                timing.roll + SCORE_ROLL_DURATION + SCORE_END_HOLD_DURATION
            }),
        }
    }
}
