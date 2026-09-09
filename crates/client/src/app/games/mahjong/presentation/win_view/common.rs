use super::super::{MahjongWinEffectTier, mahjong_win_effect_tier, mahjong_win_stage_start};
use crate::app::presentation::ease_out_cubic;
use bevy::prelude::*;
use leocard_protocol::MahjongHandResultView;

pub(super) const MAJOR_FAN_GLYPH_DELAY: f32 = 1.02;
pub(super) const MAJOR_FAN_GLYPH_INTERVAL: f32 = 0.30;

pub(crate) fn mahjong_win_effect_color(
    tier: MahjongWinEffectTier,
    secondary: bool,
    alpha: f32,
) -> Color {
    let color = match (tier, secondary) {
        (MahjongWinEffectTier::Normal, false) => Color::srgb(0.76, 1.0, 0.84),
        (MahjongWinEffectTier::Normal, true) => Color::srgb(0.35, 0.82, 0.58),
        (MahjongWinEffectTier::HighTotal, false) => Color::srgb(0.30, 1.0, 0.68),
        (MahjongWinEffectTier::HighTotal, true) => Color::srgb(0.58, 0.94, 0.84),
        (MahjongWinEffectTier::MajorFan, false) => Color::srgb(1.0, 0.76, 0.18),
        (MahjongWinEffectTier::MajorFan, true) => Color::srgb(1.0, 0.95, 0.68),
    };
    color.with_alpha(alpha)
}

pub(crate) fn mahjong_win_effect_elapsed(
    summary_elapsed: f32,
    reveal_duration: f32,
    tier: MahjongWinEffectTier,
) -> Option<f32> {
    let elapsed = summary_elapsed + reveal_duration;
    (0.0..tier.duration()).contains(&elapsed).then_some(elapsed)
}

pub(crate) fn mahjong_win_effect_visual(
    summary_elapsed: f32,
    reveal_duration: f32,
    tier: MahjongWinEffectTier,
) -> Option<(f32, f32, f32)> {
    let elapsed = mahjong_win_effect_elapsed(summary_elapsed, reveal_duration, tier)?;
    let duration = tier.duration();
    let focus = ease_out_cubic((elapsed / 0.26).clamp(0.0, 1.0));
    let impact = match tier {
        MahjongWinEffectTier::Normal => 0.72,
        MahjongWinEffectTier::HighTotal => 1.02,
        MahjongWinEffectTier::MajorFan => 1.28,
    };
    let scale = 1.0 + (1.0 - focus) * impact + (focus * std::f32::consts::PI).sin() * 0.08;
    let fade_in = (elapsed / 0.07).clamp(0.0, 1.0);
    let fade_out = ((duration - elapsed) / 0.26).clamp(0.0, 1.0);
    Some((scale, fade_in * fade_out, 9.0 * (1.0 - focus)))
}

pub(crate) fn mahjong_major_fan_impact_times(result: &MahjongHandResultView) -> Vec<f32> {
    let mut impacts = Vec::new();
    for (winner_index, winner) in result.winners.iter().enumerate() {
        if mahjong_win_effect_tier(winner) != MahjongWinEffectTier::MajorFan {
            continue;
        }
        let Some(major_fan) = winner.score.fans.iter().max_by_key(|fan| fan.fan.points()) else {
            continue;
        };
        let start = mahjong_win_stage_start(result, winner_index);
        for glyph_index in 0..major_fan.fan.name().chars().count() {
            impacts.push(
                start + MAJOR_FAN_GLYPH_DELAY + glyph_index as f32 * MAJOR_FAN_GLYPH_INTERVAL,
            );
        }
    }
    impacts
}
