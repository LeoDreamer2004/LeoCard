use super::state::{
    PlayEffectKind, PlayEffectState, SequenceAirplane, SequenceAirplaneTrail, SequenceEffectCard,
    SequenceEffectLabel, SequenceEffectLabelPart, SequenceFlowerPart, SequenceGuideSegment,
    SequenceWindStreak,
};
use crate::app::presentation::ease_out_cubic;
use bevy::prelude::*;
use std::f32::consts;

#[expect(
    clippy::type_complexity,
    reason = "the ParamSet keeps overlapping Bevy animation queries disjoint"
)]
pub(crate) fn animate_sequence_play_effect(
    effect: Res<PlayEffectState>,
    mut visuals: ParamSet<(
        Query<(&SequenceEffectCard, &mut UiTransform, &mut ImageNode)>,
        Query<(
            &SequenceGuideSegment,
            &mut UiTransform,
            &mut BackgroundColor,
        )>,
        Query<&mut UiTransform, With<SequenceEffectLabel>>,
        Query<(&SequenceEffectLabelPart, &mut TextColor)>,
        Query<(&SequenceWindStreak, &mut UiTransform, &mut BackgroundColor)>,
        Query<(&SequenceFlowerPart, &mut UiTransform, &mut BackgroundColor)>,
        Query<(&mut UiTransform, &mut ImageNode), With<SequenceAirplane>>,
        Query<(
            &SequenceAirplaneTrail,
            &mut UiTransform,
            &mut BackgroundColor,
        )>,
    )>,
) {
    let Some(active) = effect.active.as_ref() else {
        return;
    };
    let Some((_, effect_color, _)) = active.kind.sequence_style() else {
        return;
    };
    let elapsed = active.elapsed;
    let sequence_exit = if matches!(active.kind, PlayEffectKind::Airplane) {
        ((elapsed - 0.88) / 0.38).clamp(0.0, 1.0)
    } else {
        ((elapsed - 0.68) / 0.36).clamp(0.0, 1.0)
    };
    for (card, mut transform, mut image) in &mut visuals.p0() {
        let progress = ((elapsed - card.index as f32 * 0.055) / 0.24).clamp(0.0, 1.0);
        let reveal = ease_out_cubic(progress);
        transform.translation = Val2::px(-34.0 * (1.0 - reveal), -5.0 * (1.0 - reveal));
        transform.scale = Vec2::splat(0.90 + reveal * 0.10);
        // These are the real cards in the player's play area, so they stay visible
        // after the sweep instead of fading like the old table-centre duplicate did.
        image.color = Color::srgba(1.0, 1.0, 1.0, reveal);
    }
    for (segment, mut transform, mut background) in &mut visuals.p1() {
        let sweep = ((elapsed - 0.08) / 0.40).clamp(0.0, 1.0);
        let threshold = segment.index as f32 / segment.count.max(1) as f32;
        let reveal = ((sweep - threshold) * segment.count as f32).clamp(0.0, 1.0);
        let fade = ((elapsed - 0.82) / 0.28).clamp(0.0, 1.0);
        transform.scale.x = reveal;
        background.0 = effect_color.with_alpha(reveal * (1.0 - fade) * 0.92);
    }
    let reveal = ease_out_cubic(((elapsed - 0.22) / 0.22).clamp(0.0, 1.0));
    let fade = sequence_exit;
    let alpha = reveal * (1.0 - fade);
    let scale = 0.86 + reveal * 0.14;
    let base = effect_color.to_srgba();
    let outline_color = Color::srgb(
        base.red + (1.0 - base.red) * 0.32,
        base.green + (1.0 - base.green) * 0.32,
        base.blue + (1.0 - base.blue) * 0.32,
    );
    for mut transform in &mut visuals.p2() {
        transform.translation = Val2::px(12.0 + fade * 62.0, 0.0);
        transform.scale = Vec2::new(scale * 1.08, scale);
    }
    for (part, mut color) in &mut visuals.p3() {
        color.0 = if part.outline {
            outline_color.with_alpha(alpha * 0.62)
        } else {
            effect_color.with_alpha(alpha)
        };
    }

    // Match the label's exit window so the motif reads as one coherent decoration.
    let motif_exit = sequence_exit;
    let motif_fade = 1.0 - motif_exit;
    let motif_exit_x = ease_out_cubic(motif_exit) * 58.0;
    for (streak, mut transform, mut background) in &mut visuals.p4() {
        let entry = ((elapsed - 0.10 - streak.index as f32 * 0.018) / 0.16).clamp(0.0, 1.0);
        let cycle = (elapsed * 1.65 + streak.index as f32 * 0.137).fract();
        let drift = ease_out_cubic(cycle);
        let wave = (cycle * consts::TAU + streak.index as f32 * 0.9).sin();
        transform.translation = Val2::px(-18.0 + drift * 66.0 + motif_exit_x, wave * 2.8);
        transform.scale = Vec2::new(0.42 + (1.0 - cycle) * 0.72, 0.74 + wave.abs() * 0.26);
        let alpha = entry * motif_fade * (consts::PI * cycle).sin().max(0.0);
        background.0 = effect_color.with_alpha(alpha * 0.82);
    }

    for (part, mut transform, mut background) in &mut visuals.p5() {
        let delay = if part.petal {
            part.index as f32 * 0.026
        } else {
            0.08
        };
        let progress = ((elapsed - 0.14 - delay) / 0.32).clamp(0.0, 1.0);
        // A small overshoot makes each petal visibly unfold instead of merely fading in.
        let overshoot = if progress < 1.0 {
            1.0 - (1.0 - progress).powi(2) * (1.0 - progress * 1.35)
        } else {
            1.0
        };
        let pulse = 1.0 + (elapsed * 9.0 + part.index as f32).sin() * 0.035 * progress;
        if part.petal {
            let angle = part.index as f32 / 7.0 * consts::TAU - consts::FRAC_PI_2;
            let radius = 9.5 * overshoot;
            transform.translation =
                Val2::px(angle.cos() * radius + motif_exit_x, angle.sin() * radius);
            transform.rotation = Rot2::radians(angle + consts::FRAC_PI_2);
            transform.scale = Vec2::new(0.52 + overshoot * 0.48, overshoot * pulse);
            let tint = if part.index % 2 == 0 {
                Color::srgb(0.98, 0.43, 0.76)
            } else {
                Color::srgb(0.82, 0.53, 1.0)
            };
            background.0 = tint.with_alpha(progress * motif_fade * 0.95);
        } else {
            transform.translation = Val2::px(motif_exit_x, 0.0);
            transform.scale = Vec2::splat(overshoot * pulse);
            background.0 = Color::srgb(0.58, 0.96, 0.78).with_alpha(progress * motif_fade);
        }
    }

    for (mut transform, mut image) in &mut visuals.p6() {
        let progress = ((elapsed - 0.12) / 0.94).clamp(0.0, 1.0).powf(1.28);
        let pose = sequence_airplane_pose(progress);
        let appear = ((elapsed - 0.10) / 0.12).clamp(0.0, 1.0);
        transform.translation = Val2::px(pose.position.x + motif_exit_x, pose.position.y);
        transform.rotation = Rot2::radians(pose.rotation);
        transform.scale = Vec2::splat(0.50 + progress * 0.42);
        image.color = Color::WHITE.with_alpha(appear * motif_fade);
    }

    for (trail, mut transform, mut background) in &mut visuals.p7() {
        let delay = 0.055 + trail.index as f32 * 0.042;
        let progress = ((elapsed - 0.12 - delay) / 0.94).clamp(0.0, 1.0).powf(1.28);
        let pose = sequence_airplane_pose(progress);
        let direction = Vec2::new(pose.rotation.cos(), pose.rotation.sin());
        let normal = Vec2::new(-direction.y, direction.x);
        let behind = 13.0 + trail.index as f32 * 7.0;
        let lane = (trail.index as f32 - 1.5) * 1.2;
        let position = pose.position - direction * behind + normal * lane;
        let puff = ((elapsed - 0.12 - delay) / 0.18).clamp(0.0, 1.0);
        transform.translation = Val2::px(position.x + motif_exit_x, position.y);
        transform.rotation = Rot2::radians(pose.rotation);
        transform.scale = Vec2::new(0.5 + puff * 0.75, 0.45 + puff * 0.35);
        background.0 = effect_color.with_alpha(puff * motif_fade * 0.48);
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct SequenceAirplanePose {
    pub position: Vec2,
    pub rotation: f32,
}

fn sequence_airplane_pose(progress: f32) -> SequenceAirplanePose {
    let progress = progress.clamp(0.0, 1.0);
    // Start horizontally below the title, then bend around its lower-right corner.
    let p0 = Vec2::new(-6.0, 20.0);
    let p1 = Vec2::new(20.0, 20.0);
    let p2 = Vec2::new(57.0, 13.0);
    let p3 = Vec2::new(84.0, -43.0);
    let inverse = 1.0 - progress;
    let position = p0 * inverse.powi(3)
        + p1 * (3.0 * inverse.powi(2) * progress)
        + p2 * (3.0 * inverse * progress.powi(2))
        + p3 * progress.powi(3);
    let tangent = (p1 - p0) * (3.0 * inverse.powi(2))
        + (p2 - p1) * (6.0 * inverse * progress)
        + (p3 - p2) * (3.0 * progress.powi(2));
    SequenceAirplanePose {
        position,
        rotation: tangent.y.atan2(tangent.x),
    }
}
