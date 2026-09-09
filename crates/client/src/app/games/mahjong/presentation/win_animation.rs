use super::win_view::{
    mahjong_win_effect_color, mahjong_win_effect_elapsed, mahjong_win_effect_visual,
};
use super::{
    MAHJONG_WIN_PUSH_DURATION, MahjongWinDecoration, MahjongWinDecorationKind, MahjongWinEffect,
    MahjongWinEffectText, MahjongWinEffectTier, MahjongWinFanGlyph, MahjongWinScreenShake,
    MahjongWinStageKind, MahjongWinStagePart, MahjongWinningHand,
};
use crate::app::presentation::{DESIGN_WIDTH, GameSummaryAnimation, ease_out_cubic};
use bevy::prelude::*;

pub(crate) fn animate_mahjong_win_effects(
    animation: Res<GameSummaryAnimation>,
    mut effects: Query<
        (&MahjongWinEffect, &mut UiTransform, &mut Visibility),
        (
            Without<MahjongWinDecoration>,
            Without<MahjongWinStagePart>,
            Without<MahjongWinFanGlyph>,
        ),
    >,
    mut decorations: Query<
        (
            &MahjongWinDecoration,
            &mut UiTransform,
            &mut Visibility,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        (
            Without<MahjongWinEffect>,
            Without<MahjongWinStagePart>,
            Without<MahjongWinFanGlyph>,
        ),
    >,
    mut texts: Query<
        (&MahjongWinEffectText, &mut TextColor, &mut TextShadow),
        (Without<MahjongWinStagePart>, Without<MahjongWinFanGlyph>),
    >,
    mut glyphs: Query<
        (
            &MahjongWinFanGlyph,
            &mut UiTransform,
            &mut Visibility,
            &mut TextColor,
            &mut TextShadow,
        ),
        (
            Without<MahjongWinEffect>,
            Without<MahjongWinDecoration>,
            Without<MahjongWinEffectText>,
            Without<MahjongWinStagePart>,
        ),
    >,
    mut stages: Query<
        (
            &MahjongWinStagePart,
            &mut UiTransform,
            &mut Visibility,
            Option<&mut BackgroundColor>,
            Option<&mut BorderColor>,
        ),
        (
            Without<MahjongWinEffect>,
            Without<MahjongWinDecoration>,
            Without<MahjongWinEffectText>,
            Without<MahjongWinFanGlyph>,
        ),
    >,
) {
    for (effect, mut transform, mut visibility) in &mut effects {
        let Some((scale, _, offset_y)) =
            mahjong_win_effect_visual(animation.elapsed, effect.reveal_duration, effect.tier)
        else {
            *visibility = Visibility::Hidden;
            continue;
        };
        transform.translation = Val2::px(0.0, offset_y);
        transform.scale = Vec2::splat(scale);
        *visibility = Visibility::Visible;
    }
    for (effect, mut color, mut shadow) in &mut texts {
        let Some((_, alpha, _)) =
            mahjong_win_effect_visual(animation.elapsed, effect.reveal_duration, effect.tier)
        else {
            color.0 = Color::NONE;
            shadow.color = Color::NONE;
            continue;
        };
        color.0 = mahjong_win_effect_color(effect.tier, false, alpha);
        shadow.color = Color::BLACK.with_alpha(0.76 * alpha);
    }
    for (glyph, mut transform, mut visibility, mut color, mut shadow) in &mut glyphs {
        const LIFETIME: f32 = 1.24;
        let local = animation.elapsed + glyph.reveal_duration - glyph.start - glyph.delay;
        if !(0.0..LIFETIME).contains(&local) {
            *visibility = Visibility::Hidden;
            color.0 = Color::NONE;
            shadow.color = Color::NONE;
            continue;
        }
        let impact = ease_out_cubic((local / 0.18).clamp(0.0, 1.0));
        let vibration = ((local - 0.12) / 0.42).clamp(0.0, 1.0);
        let strength = (1.0 - vibration).powi(2);
        let fade_out = ((LIFETIME - local) / 0.28).clamp(0.0, 1.0);
        transform.translation = Val2::px(
            (local * 93.0).sin() * 11.0 * strength,
            -190.0 * (1.0 - impact) + (local * 127.0).sin() * 6.0 * strength,
        );
        transform.rotation = Rot2::radians((local * 71.0).sin() * 0.075 * strength);
        transform.scale = Vec2::splat(
            1.0 + 3.4 * (1.0 - impact) + (vibration * std::f32::consts::PI).sin() * 0.12,
        );
        color.0 = mahjong_win_effect_color(MahjongWinEffectTier::MajorFan, false, fade_out);
        shadow.color = Color::BLACK.with_alpha(0.88 * fade_out);
        *visibility = Visibility::Visible;
    }
    for (decoration, mut transform, mut visibility, mut background, mut border) in &mut decorations
    {
        let Some(elapsed) = mahjong_win_effect_elapsed(
            animation.elapsed,
            decoration.reveal_duration,
            decoration.tier,
        ) else {
            *visibility = Visibility::Hidden;
            continue;
        };
        let duration = decoration.tier.duration();
        let (translation, scale, rotation, alpha, secondary, ring) = match decoration.kind {
            MahjongWinDecorationKind::Halo => {
                let progress = ease_out_cubic((elapsed / duration).clamp(0.0, 1.0));
                let fade_in = (elapsed / 0.10).clamp(0.0, 1.0);
                let fade_out = ((duration - elapsed) / 0.30).clamp(0.0, 1.0);
                let strength = match decoration.tier {
                    MahjongWinEffectTier::Normal => 0.13,
                    MahjongWinEffectTier::HighTotal => 0.20,
                    MahjongWinEffectTier::MajorFan => 0.28,
                };
                (
                    Vec2::ZERO,
                    0.72 + progress * 0.56,
                    0.0,
                    fade_in * fade_out * strength,
                    false,
                    false,
                )
            }
            MahjongWinDecorationKind::Ring {
                delay,
                start_scale,
                end_scale,
                max_alpha,
            } => {
                let progress = ((elapsed - delay) / (duration - delay).max(0.01)).clamp(0.0, 1.0);
                let motion = ease_out_cubic(progress);
                (
                    Vec2::ZERO,
                    start_scale + (end_scale - start_scale) * motion,
                    0.0,
                    (progress * std::f32::consts::PI).sin().max(0.0) * max_alpha,
                    delay > 0.0,
                    true,
                )
            }
            MahjongWinDecorationKind::Ray {
                direction,
                distance,
                delay,
                secondary,
            } => {
                let progress = ((elapsed - delay) / 0.55).clamp(0.0, 1.0);
                let motion = ease_out_cubic(progress);
                (
                    direction * distance * motion,
                    0.55 + motion * 0.65,
                    direction.y.atan2(direction.x),
                    (progress * std::f32::consts::PI).sin().max(0.0) * 0.82,
                    secondary,
                    false,
                )
            }
        };
        transform.translation = Val2::px(translation.x, translation.y);
        transform.scale = Vec2::splat(scale);
        transform.rotation = Rot2::radians(rotation);
        let color = mahjong_win_effect_color(decoration.tier, secondary, alpha);
        if ring {
            background.0 = Color::NONE;
            *border = BorderColor::all(color);
        } else {
            background.0 = color;
            *border = BorderColor::all(Color::NONE);
        }
        *visibility = if alpha > 0.0 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    for (stage, mut transform, mut visibility, background, border) in &mut stages {
        let elapsed = animation.elapsed + stage.reveal_duration - stage.start;
        if !(0.0..stage.duration).contains(&elapsed) {
            *visibility = Visibility::Hidden;
            continue;
        }
        *visibility = Visibility::Visible;
        let fade_out = ((stage.duration - elapsed) / 0.18).clamp(0.0, 1.0);
        match stage.kind {
            MahjongWinStageKind::Backdrop => {
                let fade_in = (elapsed / 0.16).clamp(0.0, 1.0);
                let strength = if stage.tier == MahjongWinEffectTier::MajorFan {
                    0.76
                } else {
                    0.44
                };
                transform.translation = Val2::ZERO;
                transform.scale = Vec2::ONE;
                transform.rotation = Rot2::IDENTITY;
                if let Some(mut background) = background {
                    background.0 = if stage.tier == MahjongWinEffectTier::MajorFan {
                        Color::BLACK.with_alpha(fade_in * fade_out * strength)
                    } else {
                        Color::BLACK.with_alpha(fade_in * fade_out * 0.86)
                    };
                }
            }
            MahjongWinStageKind::Hand => {
                let progress = ease_out_cubic((elapsed / 0.78).clamp(0.0, 1.0));
                transform.translation = Val2::px(0.0, -12.0 * (1.0 - progress));
                transform.scale = Vec2::new(1.0, 0.08 + progress * 0.92);
                transform.rotation = Rot2::IDENTITY;
                if let Some(mut background) = background {
                    background.0 = Color::NONE;
                }
            }
            MahjongWinStageKind::WinningTile => {
                if stage.tier == MahjongWinEffectTier::MajorFan {
                    let decay = (1.0 - (elapsed / stage.duration).clamp(0.0, 1.0)).powi(2);
                    transform.translation = Val2::px(
                        (elapsed * 91.0).sin() * 15.0 * decay,
                        (elapsed * 137.0).sin() * 8.0 * decay,
                    );
                    transform.rotation = Rot2::radians(
                        ((elapsed * 61.0).sin() * 0.20 + (elapsed * 29.0).sin() * 0.08) * decay,
                    );
                    transform.scale = Vec2::ONE;
                } else {
                    let progress = ease_out_cubic((elapsed / 0.30).clamp(0.0, 1.0));
                    let impact = if stage.tier == MahjongWinEffectTier::HighTotal {
                        0.82
                    } else {
                        0.52
                    };
                    transform.translation = Val2::px(0.0, -16.0 * progress);
                    transform.rotation = Rot2::radians((elapsed * 18.0).sin() * 0.035);
                    transform.scale = Vec2::splat(1.0 + (1.0 - progress) * impact);
                }
            }
            MahjongWinStageKind::FocusRay {
                delay,
                direction,
                phase,
            } => {
                let local = elapsed - delay;
                if local < 0.0 {
                    *visibility = Visibility::Hidden;
                    continue;
                }
                let intro = ease_out_cubic((local / 0.24).clamp(0.0, 1.0));
                let cycle = (local * 0.72 + phase).fract();
                let envelope = (cycle * std::f32::consts::PI).sin().max(0.0).powf(0.65);
                let travel = 72.0 - cycle * 112.0;
                transform.translation = Val2::px(direction.x * travel, direction.y * travel);
                transform.rotation = Rot2::radians(direction.y.atan2(direction.x));
                transform.scale = Vec2::new(0.42 + envelope * 0.86, 0.76 + envelope * 0.24);
                if let Some(mut background) = background {
                    background.0 = mahjong_win_effect_color(
                        MahjongWinEffectTier::HighTotal,
                        phase > 0.48,
                        intro * fade_out * envelope * 0.78,
                    );
                }
            }
            MahjongWinStageKind::MajorFrame => {
                let progress = ease_out_cubic((elapsed / 0.48).clamp(0.0, 1.0));
                let pulse = 0.76 + (elapsed * 3.4).sin() * 0.14;
                transform.translation = Val2::ZERO;
                transform.rotation = Rot2::IDENTITY;
                transform.scale = Vec2::splat(1.018 - progress * 0.018);
                if let Some(mut border) = border {
                    *border = BorderColor::all(mahjong_win_effect_color(
                        MahjongWinEffectTier::MajorFan,
                        true,
                        progress * fade_out * pulse,
                    ));
                }
            }
            MahjongWinStageKind::MajorSweep { delay } => {
                let local = elapsed - delay;
                if local < 0.0 {
                    *visibility = Visibility::Hidden;
                    continue;
                }
                let progress = ease_out_cubic((local / 0.56).clamp(0.0, 1.0));
                let shimmer = 0.64 + (local * 4.2).sin().abs() * 0.24;
                transform.translation = Val2::ZERO;
                transform.rotation = Rot2::IDENTITY;
                transform.scale = Vec2::new(0.02 + progress * 0.98, 1.0);
                if let Some(mut background) = background {
                    background.0 = mahjong_win_effect_color(
                        MahjongWinEffectTier::MajorFan,
                        true,
                        progress * fade_out * shimmer,
                    );
                }
            }
            MahjongWinStageKind::MajorSpark {
                delay,
                drift,
                phase,
            } => {
                let local = elapsed - delay;
                if local < 0.0 {
                    *visibility = Visibility::Hidden;
                    continue;
                }
                let cycle = (local * 0.43 + phase).fract();
                let glow = (cycle * std::f32::consts::PI).sin().max(0.0);
                transform.translation = Val2::px(drift.x * cycle, drift.y * cycle);
                transform.rotation = Rot2::radians(local * 1.8 + phase * 5.0);
                transform.scale = Vec2::splat(0.55 + glow * 0.85);
                if let Some(mut background) = background {
                    background.0 = mahjong_win_effect_color(
                        MahjongWinEffectTier::MajorFan,
                        phase > 0.5,
                        glow * fade_out * 0.74,
                    );
                }
            }
            MahjongWinStageKind::ImpactFlash { delay } => {
                let local = elapsed - delay;
                if !(0.0..0.26).contains(&local) {
                    *visibility = Visibility::Hidden;
                    continue;
                }
                let strength = (local / 0.26 * std::f32::consts::PI).sin().max(0.0);
                transform.translation = Val2::ZERO;
                transform.rotation = Rot2::IDENTITY;
                transform.scale = Vec2::ONE;
                if let Some(mut background) = background {
                    background.0 = Color::srgba(1.0, 0.72, 0.16, strength * 0.13);
                }
            }
        }
    }
}

pub(crate) fn animate_mahjong_win_screen_shake(
    animation: Res<GameSummaryAnimation>,
    mut tables: Query<(&MahjongWinScreenShake, &mut UiTransform)>,
) {
    for (shake, mut transform) in &mut tables {
        let timeline = animation.elapsed + shake.reveal_duration;
        let mut offset = Vec2::ZERO;
        let mut rotation = 0.0;
        for impact in &shake.impacts {
            let local = timeline - impact;
            if !(0.0..0.38).contains(&local) {
                continue;
            }
            let strength = (1.0 - local / 0.38).powi(2);
            offset.x += (local * 109.0).sin() * 5.5 * strength;
            offset.y += (local * 151.0).sin() * 3.5 * strength;
            rotation += (local * 83.0).sin() * 0.006 * strength;
        }
        transform.translation = Val2::px(-DESIGN_WIDTH / 2.0 + offset.x, offset.y);
        transform.rotation = Rot2::radians(rotation);
    }
}

pub(crate) fn mahjong_winning_hand_progress(
    summary_elapsed: f32,
    reveal_duration: f32,
    start: f32,
) -> f32 {
    ((summary_elapsed + reveal_duration - start) / MAHJONG_WIN_PUSH_DURATION).clamp(0.0, 1.0)
}

pub(crate) fn apply_mahjong_winning_hand_visual(
    transform: &mut UiTransform,
    relative: u8,
    base_rotation: f32,
    progress: f32,
) {
    let progress = ease_out_cubic(progress);
    if relative == 0 {
        transform.translation = Val2::px(0.0, 8.0 * (1.0 - progress));
        transform.rotation = Rot2::radians(base_rotation);
        transform.scale = Vec2::splat(0.97 + progress * 0.03);
        return;
    }
    let distance = 12.0 * (1.0 - progress);
    let offset = match relative {
        0 => Vec2::new(0.0, distance),
        1 => Vec2::new(distance, 0.0),
        2 => Vec2::new(0.0, -distance),
        _ => Vec2::new(-distance, 0.0),
    };
    transform.translation = Val2::px(offset.x, offset.y);
    transform.rotation = Rot2::radians(base_rotation);
    transform.scale = Vec2::new(0.96 + progress * 0.04, 0.02 + progress * 0.98);
}

pub(crate) fn animate_mahjong_winning_hands(
    animation: Res<GameSummaryAnimation>,
    mut hands: Query<(&MahjongWinningHand, &mut UiTransform, &mut Visibility)>,
) {
    for (hand, mut transform, mut visibility) in &mut hands {
        let progress =
            mahjong_winning_hand_progress(animation.elapsed, hand.reveal_duration, hand.start);
        *visibility = if hand.relative == 0 || progress > 0.0 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        apply_mahjong_winning_hand_visual(
            &mut transform,
            hand.relative,
            hand.base_rotation,
            progress,
        );
    }
}
