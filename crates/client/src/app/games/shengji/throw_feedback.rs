//! 甩牌失败、退牌与罚分演出。

use super::*;

const SHENGJI_FAILED_THROW_RETURN_DURATION: f32 = 0.42;
const SHENGJI_THROW_PENALTY_DELAY: f32 = 0.38;
const SHENGJI_THROW_PENALTY_DURATION: f32 = 0.58;

#[derive(Clone, Copy, Debug)]
pub struct ShengjiFailedThrowCardVisual {
    pub translation: Vec2,
    pub scale: f32,
    pub rotation_radians: f32,
    pub visible: bool,
}

pub fn shengji_failed_throw_card_visual(
    stage: ShengjiThrowFailureStage,
    index: usize,
    count: usize,
    elapsed: f32,
    return_direction: Vec2,
) -> ShengjiFailedThrowCardVisual {
    let spread = index as f32 - count.saturating_sub(1) as f32 * 0.5;
    match stage {
        ShengjiThrowFailureStage::Showing => {
            // 先由一叠牌完整展开；停顿片刻后向两边裂开，并弹回原位。
            let reveal = ease_out_cubic((elapsed / 0.20).clamp(0.0, 1.0));
            let crack_progress = ((elapsed - 0.30) / 0.48).clamp(0.0, 1.0);
            let crack = (crack_progress * std::f32::consts::PI).sin();
            let side = spread.signum();
            let alternate = if index.is_multiple_of(2) { -1.0 } else { 1.0 };
            ShengjiFailedThrowCardVisual {
                translation: Vec2::new(
                    -spread * 24.0 * (1.0 - reveal) + side * (9.0 + spread.abs() * 1.8) * crack,
                    (alternate * 5.0 - 4.0) * crack,
                ),
                scale: 0.94 + reveal * 0.06 + crack * 0.035,
                rotation_radians: (spread * 1.6 * crack).to_radians(),
                visible: true,
            }
        }
        ShengjiThrowFailureStage::Returning => {
            let stagger = index as f32 / count.max(1) as f32 * 0.07;
            let raw = ((elapsed - stagger) / (SHENGJI_FAILED_THROW_RETURN_DURATION - 0.07))
                .clamp(0.0, 1.0);
            let return_progress = raw * raw * raw;
            let rebound = 1.0 - ease_out_cubic((raw / 0.34).clamp(0.0, 1.0));
            let alternate = if index.is_multiple_of(2) { -1.0 } else { 1.0 };
            let split = Vec2::new(spread.signum() * (7.0 + spread.abs()), alternate * 4.0);
            ShengjiFailedThrowCardVisual {
                translation: split * rebound + return_direction * return_progress,
                scale: 1.0 - return_progress * 0.22,
                rotation_radians: (spread * 1.1 * rebound).to_radians(),
                visible: raw < 1.0,
            }
        }
    }
}

pub fn animate_shengji_failed_throw_cards(
    time: Res<Time>,
    mut cards: Query<(
        &mut ShengjiFailedThrowCard,
        &mut UiTransform,
        &mut Visibility,
    )>,
) {
    for (mut animation, mut transform, mut visibility) in &mut cards {
        animation.elapsed += time.delta_secs();
        let visual = shengji_failed_throw_card_visual(
            animation.stage,
            animation.index,
            animation.count,
            animation.elapsed,
            animation.direction,
        );
        transform.translation = Val2::px(visual.translation.x, visual.translation.y);
        transform.scale = Vec2::splat(visual.scale);
        transform.rotation = Rot2::radians(visual.rotation_radians);
        *visibility = if visual.visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

pub fn animate_shengji_failed_throw_labels(
    time: Res<Time>,
    mut labels: Query<(
        &mut ShengjiFailedThrowLabel,
        &mut UiTransform,
        &mut TextColor,
        &mut Visibility,
    )>,
) {
    for (mut animation, mut transform, mut color, mut visibility) in &mut labels {
        animation.elapsed += time.delta_secs();
        if animation.returning {
            let progress = ease_out_cubic((animation.elapsed / 0.30).clamp(0.0, 1.0));
            transform.translation = Val2::px(0.0, -12.0 * progress);
            transform.scale = Vec2::splat(1.0 - 0.12 * progress);
            color.0 = DANGER.with_alpha(1.0 - progress);
            if progress >= 1.0 {
                *visibility = Visibility::Hidden;
            }
        } else {
            let enter = ease_out_cubic((animation.elapsed / 0.16).clamp(0.0, 1.0));
            let shake_progress = ((animation.elapsed - 0.24) / 0.42).clamp(0.0, 1.0);
            let shake =
                (shake_progress * std::f32::consts::TAU * 3.0).sin() * (1.0 - shake_progress) * 3.0;
            transform.translation = Val2::px(shake, 7.0 * (1.0 - enter));
            transform.scale = Vec2::splat(0.86 + 0.14 * enter);
            color.0 = DANGER.with_alpha(enter);
            *visibility = Visibility::Visible;
        }
    }
}

pub fn animate_shengji_throw_penalty_floats(
    time: Res<Time>,
    mut penalties: Query<(
        &mut ShengjiThrowPenaltyFloat,
        &mut Node,
        &mut UiTransform,
        &mut TextColor,
        &mut Visibility,
    )>,
) {
    for (mut animation, mut node, mut transform, mut color, mut visibility) in &mut penalties {
        animation.elapsed += time.delta_secs();
        let raw = ((animation.elapsed - SHENGJI_THROW_PENALTY_DELAY)
            / SHENGJI_THROW_PENALTY_DURATION)
            .clamp(0.0, 1.0);
        let travel = ease_out_cubic(raw);
        let position = animation.source.lerp(animation.target, travel);
        node.left = percent(position.x);
        node.top = percent(position.y);
        let arc = (raw * std::f32::consts::PI).sin();
        transform.translation = Val2::px(-30.0, -18.0 - arc * 20.0);
        transform.scale = Vec2::splat(0.86 + arc * 0.20);
        color.0 =
            DANGER.with_alpha((raw / 0.12).clamp(0.0, 1.0) * ((1.0 - raw) / 0.14).clamp(0.0, 1.0));
        *visibility = if animation.elapsed < SHENGJI_THROW_PENALTY_DELAY || raw >= 1.0 {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };
    }
}

pub fn animate_shengji_throw_penalty_score_pulses(
    time: Res<Time>,
    mut scores: Query<(&mut ShengjiThrowPenaltyScorePulse, &mut UiTransform)>,
) {
    for (mut animation, mut transform) in &mut scores {
        animation.elapsed += time.delta_secs();
        let progress = ((animation.elapsed
            - SHENGJI_THROW_PENALTY_DELAY
            - SHENGJI_THROW_PENALTY_DURATION * 0.72)
            / 0.30)
            .clamp(0.0, 1.0);
        transform.scale = Vec2::splat(1.0 + (progress * std::f32::consts::PI).sin() * 0.18);
    }
}
