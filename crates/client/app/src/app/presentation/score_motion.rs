use super::{SCORE_ROLL_DELAY, SCORE_ROLL_DURATION, ease_out_cubic};
use bevy::prelude::*;

pub(crate) const SCORE_GAIN_DURATION: f32 = SCORE_ROLL_DELAY + SCORE_ROLL_DURATION + 0.30;

pub(crate) fn rolling_score(before: u32, after: u32, elapsed: f32) -> u32 {
    let progress = ((elapsed - SCORE_ROLL_DELAY) / SCORE_ROLL_DURATION).clamp(0.0, 1.0);
    (before as f32 + after.saturating_sub(before) as f32 * ease_out_cubic(progress)).round() as u32
}

pub(crate) fn score_gain_visual(elapsed: f32) -> (UiTransform, f32) {
    let local = elapsed - SCORE_ROLL_DELAY;
    let progress = (local / (SCORE_ROLL_DURATION + 0.30)).clamp(0.0, 1.0);
    let alpha = if local < 0.0 {
        0.0
    } else {
        ((SCORE_ROLL_DURATION + 0.30 - local) / 0.30).clamp(0.0, 1.0)
    };
    (
        UiTransform {
            translation: Val2::px(0.0, -22.0 * ease_out_cubic(progress)),
            scale: Vec2::splat(0.82 + 0.18 * ease_out_cubic((progress * 4.0).min(1.0))),
            ..default()
        },
        alpha,
    )
}
