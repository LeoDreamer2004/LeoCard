use super::*;

pub fn animate_mahjong_win_effects(
    animation: Res<GameSummaryAnimation>,
    mut effects: Query<
        (&MahjongWinEffect, &mut UiTransform, &mut Visibility),
        (Without<MahjongWinDecoration>, Without<MahjongWinStagePart>),
    >,
    mut decorations: Query<
        (
            &MahjongWinDecoration,
            &mut UiTransform,
            &mut Visibility,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        (Without<MahjongWinEffect>, Without<MahjongWinStagePart>),
    >,
    mut texts: Query<
        (&MahjongWinEffectText, &mut TextColor, &mut TextShadow),
        Without<MahjongWinStagePart>,
    >,
    mut stages: Query<
        (
            &MahjongWinStagePart,
            &mut UiTransform,
            &mut Visibility,
            Option<&mut BackgroundColor>,
        ),
        (
            Without<MahjongWinEffect>,
            Without<MahjongWinDecoration>,
            Without<MahjongWinEffectText>,
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
    for (stage, mut transform, mut visibility, background) in &mut stages {
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
                        Color::srgb(0.01, 0.12, 0.09).with_alpha(fade_in * fade_out * strength)
                    };
                }
            }
            MahjongWinStageKind::Hand => {
                let progress = ease_out_cubic((elapsed / 0.78).clamp(0.0, 1.0));
                transform.translation = Val2::px(0.0, 18.0 * (1.0 - progress));
                transform.scale = Vec2::new(0.02 + progress * 0.98, 0.96 + progress * 0.04);
                transform.rotation = Rot2::IDENTITY;
                if let Some(mut background) = background {
                    background.0 = if stage.tier == MahjongWinEffectTier::MajorFan {
                        Color::NONE
                    } else {
                        Color::BLACK.with_alpha(0.86 * fade_out)
                    };
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
                    transform.scale = Vec2::splat(1.72 + (elapsed * 44.0).sin().abs() * 0.10);
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
            MahjongWinStageKind::FanText { delay } => {
                if elapsed < delay {
                    *visibility = Visibility::Hidden;
                    continue;
                }
                let local = elapsed - delay;
                let impact = ease_out_cubic((local / 0.18).clamp(0.0, 1.0));
                let vibration = ((local - 0.12) / 0.42).clamp(0.0, 1.0);
                let strength = (1.0 - vibration).powi(2);
                transform.translation = Val2::px(
                    (local * 93.0).sin() * 11.0 * strength,
                    -190.0 * (1.0 - impact) + (local * 127.0).sin() * 6.0 * strength,
                );
                transform.rotation = Rot2::radians((local * 71.0).sin() * 0.075 * strength);
                transform.scale = Vec2::splat(
                    1.0 + 3.4 * (1.0 - impact) + (vibration * std::f32::consts::PI).sin() * 0.12,
                );
            }
        }
    }
}

pub fn animate_mahjong_win_screen_shake(
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

pub fn mahjong_winning_hand_progress(
    summary_elapsed: f32,
    reveal_duration: f32,
    start: f32,
) -> f32 {
    ((summary_elapsed + reveal_duration - start) / MAHJONG_WIN_PUSH_DURATION).clamp(0.0, 1.0)
}

pub fn apply_mahjong_winning_hand_visual(
    transform: &mut UiTransform,
    relative: u8,
    base_rotation: f32,
    progress: f32,
) {
    let progress = ease_out_cubic(progress);
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

pub fn animate_mahjong_winning_hands(
    animation: Res<GameSummaryAnimation>,
    mut hands: Query<(&MahjongWinningHand, &mut UiTransform, &mut Visibility)>,
) {
    for (hand, mut transform, mut visibility) in &mut hands {
        let progress =
            mahjong_winning_hand_progress(animation.elapsed, hand.reveal_duration, hand.start);
        *visibility = if progress > 0.0 {
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
