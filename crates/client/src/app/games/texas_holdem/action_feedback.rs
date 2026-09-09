//! 德州扑克下注动作的短促视觉反馈，与筹码账本和牌桌构建解耦。

use super::{ActionFeedbackKind, TexasChipTableState, TexasPlayerPanel};
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use std::collections::HashSet;

#[derive(Component)]
pub(super) struct TexasActionFeedback {
    pub kind: ActionFeedbackKind,
    pub elapsed: f32,
}

#[derive(Component)]
pub(super) struct TexasActionFeedbackText {
    pub color: Color,
    pub kind: ActionFeedbackKind,
    pub elapsed: f32,
}

#[derive(Component)]
pub(super) struct TexasFoldCard {
    pub index: usize,
    pub total: usize,
    pub elapsed: f32,
    pub own: bool,
    pub face: Option<Handle<Image>>,
    pub back: Handle<Image>,
}

#[derive(Component)]
pub(super) struct TexasOwnFoldCardHover {
    pub tooltip: Entity,
}

#[derive(Component)]
pub(super) struct TexasOwnFoldTooltip;

#[derive(Clone, Copy, Debug)]
pub(crate) struct ActionFeedbackVisual {
    pub translation: Vec2,
    pub scale: f32,
    pub alpha: f32,
}

pub(super) fn action_feedback_visual(
    kind: ActionFeedbackKind,
    elapsed: f32,
) -> ActionFeedbackVisual {
    let entry = (elapsed / 0.18).clamp(0.0, 1.0);
    let mut visual = ActionFeedbackVisual {
        translation: Vec2::ZERO,
        scale: 0.84 + entry * 0.16,
        alpha: entry,
    };
    match kind {
        ActionFeedbackKind::Blind => {}
        ActionFeedbackKind::Fold => {
            visual.scale = 0.90 + entry * 0.10;
        }
        ActionFeedbackKind::Check => {}
        ActionFeedbackKind::Call => {
            visual.scale = 1.0 + (entry * std::f32::consts::PI).sin() * 0.12;
        }
        ActionFeedbackKind::Raise => {
            let progress = (elapsed / 0.28).clamp(0.0, 1.0);
            visual.scale = 0.78 + 1.22 * progress - progress * progress;
        }
        ActionFeedbackKind::AllIn => {
            let progress = (elapsed / 0.34).clamp(0.0, 1.0);
            visual.scale = 0.72 + 1.40 * progress - 1.12 * progress * progress;
            let shake = 1.0 - (elapsed / 0.48).clamp(0.0, 1.0);
            visual.translation.x = (elapsed * 56.0).sin() * 5.5 * shake;
        }
    }
    visual
}

pub(super) fn action_feedback_transform(kind: ActionFeedbackKind, elapsed: f32) -> UiTransform {
    let visual = action_feedback_visual(kind, elapsed);
    let mut transform = UiTransform::IDENTITY;
    transform.translation = Val2::px(visual.translation.x, visual.translation.y);
    transform.scale = Vec2::splat(visual.scale);
    transform
}

pub(super) fn action_feedback_text_color(
    kind: ActionFeedbackKind,
    base: Color,
    elapsed: f32,
) -> Color {
    let alpha = action_feedback_visual(kind, elapsed).alpha;
    if kind != ActionFeedbackKind::Check {
        return base.with_alpha(alpha);
    }

    // “过牌”不从桌面消失，只从正文白色平滑过渡到不抢眼的灰色。
    let progress = ((elapsed - 0.22) / 0.42).clamp(0.0, 1.0);
    let eased = progress * progress * (3.0 - 2.0 * progress);
    let start = Vec3::new(0.94, 0.97, 0.95);
    let end = Vec3::new(0.50, 0.55, 0.53);
    let rgb = start.lerp(end, eased);
    Color::srgba(rgb.x, rgb.y, rgb.z, alpha)
}

pub(crate) struct FoldCardVisual {
    pub transform: UiTransform,
    pub face_visible: bool,
}

pub(super) fn fold_card_visual(
    index: usize,
    total: usize,
    elapsed: f32,
    own: bool,
) -> FoldCardVisual {
    let duration = if own { 0.72 } else { 0.58 };
    let progress = (elapsed / duration).clamp(0.0, 1.0);
    let eased = 1.0 - (1.0 - progress).powi(3);
    let centered_index = index as f32 - total.saturating_sub(1) as f32 / 2.0;
    let side = if centered_index < 0.0 { -1.0 } else { 1.0 };
    let mut transform = UiTransform::IDENTITY;
    if own {
        let (start_x, end_x) = if total > 2 {
            (-138.0 + index as f32 * 66.0, -26.0 + index as f32 * 8.0)
        } else {
            (
                if index == 0 { -88.0 } else { 5.0 },
                if index == 0 { -18.0 } else { -10.0 },
            )
        };
        let start = Vec2::new(start_x, 110.0);
        let end = Vec2::new(end_x, 6.0);
        let position = start.lerp(end, eased);
        let base_scale = 2.42 + (0.76 - 2.42) * eased;
        let flip = (progress / 0.34).clamp(0.0, 1.0);
        let horizontal = (1.0 - flip * 2.0).abs().max(0.035);
        transform.translation = Val2::px(position.x, position.y);
        transform.rotation = Rot2::radians(side * 0.055 * (1.0 - eased));
        transform.scale = Vec2::new(base_scale * horizontal, base_scale);
        FoldCardVisual {
            transform,
            face_visible: flip < 0.5,
        }
    } else {
        let start_spacing = if total > 2 { 34.0 } else { 104.0 };
        let rotation_spacing = if total > 2 { 0.10 } else { 0.32 };
        let start_x = -14.0 + centered_index * start_spacing;
        let end_x = -14.0 + centered_index * 8.0;
        transform.translation = Val2::px(start_x + (end_x - start_x) * eased, 6.0 * eased);
        transform.rotation = Rot2::radians(centered_index * rotation_spacing * (1.0 - eased));
        transform.scale = Vec2::splat(1.0 - eased * 0.24);
        FoldCardVisual {
            transform,
            face_visible: false,
        }
    }
}

pub(super) fn animate_texas_action_feedback(
    time: Res<Time>,
    state: Res<TexasChipTableState>,
    mut visuals: ParamSet<(
        Query<(&mut TexasActionFeedback, &mut UiTransform, &mut Visibility)>,
        Query<(&mut TexasFoldCard, &mut UiTransform, &mut ImageNode)>,
        Query<(&TexasPlayerPanel, &mut UiTransform)>,
    )>,
    mut texts: Query<(&mut TexasActionFeedbackText, &mut TextColor)>,
) {
    for (mut feedback, mut transform, mut visibility) in &mut visuals.p0() {
        feedback.elapsed += time.delta_secs();
        let visual = action_feedback_visual(feedback.kind, feedback.elapsed);
        *transform = action_feedback_transform(feedback.kind, feedback.elapsed);
        *visibility = if visual.alpha > 0.001 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    for (mut style, mut color) in &mut texts {
        style.elapsed += time.delta_secs();
        color.0 = action_feedback_text_color(style.kind, style.color, style.elapsed);
    }
    for (mut card, mut transform, mut image) in &mut visuals.p1() {
        card.elapsed += time.delta_secs();
        let visual = fold_card_visual(card.index, card.total, card.elapsed, card.own);
        *transform = visual.transform;
        image.image = if visual.face_visible {
            card.face.as_ref().unwrap_or(&card.back).clone()
        } else {
            card.back.clone()
        };
        image.color = Color::WHITE;
    }
    for (panel, mut transform) in &mut visuals.p2() {
        let shake_x = state
            .actions
            .get(&panel.player)
            .filter(|label| label.kind == ActionFeedbackKind::AllIn)
            .map_or(0.0, |label| {
                action_feedback_visual(label.kind, label.elapsed)
                    .translation
                    .x
            });
        transform.translation.x = px(shake_x);
    }
}

pub(super) fn sync_texas_own_fold_tooltip(
    hovers: Query<(&RelativeCursorPosition, &TexasOwnFoldCardHover)>,
    mut tooltips: Query<(Entity, &mut Visibility), With<TexasOwnFoldTooltip>>,
) {
    let hovered = hovers
        .iter()
        .filter(|(cursor, _)| cursor.cursor_over())
        .map(|(_, hover)| hover.tooltip)
        .collect::<HashSet<_>>();
    for (entity, mut visibility) in &mut tooltips {
        *visibility = if hovered.contains(&entity) {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}
