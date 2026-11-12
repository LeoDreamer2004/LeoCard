use super::super::super::MahjongWinEffectTier;
use crate::app::presentation::ease_out_cubic;
use std::f32::consts;

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
    let scale = 1.0 + (1.0 - focus) * 0.18 + (focus * consts::PI).sin() * 0.025;
    let fade_in = (elapsed / 0.07).clamp(0.0, 1.0);
    let fade_out = ((duration - elapsed) / 0.26).clamp(0.0, 1.0);
    Some((scale, fade_in * fade_out, 9.0 * (1.0 - focus)))
}
