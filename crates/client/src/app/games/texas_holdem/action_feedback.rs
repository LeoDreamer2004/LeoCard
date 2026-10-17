//! 德州扑克下注动作的短促视觉反馈，与筹码账本和牌桌构建解耦。

use super::{ActionFeedbackKind, TexasChipTableState, TexasPlayerShake};
use crate::app::presentation::TableStatusEffect;
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use std::collections::HashSet;

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

impl From<ActionFeedbackKind> for TableStatusEffect {
    fn from(kind: ActionFeedbackKind) -> Self {
        match kind {
            ActionFeedbackKind::Blind => Self::Appear,
            ActionFeedbackKind::Fold => Self::Settle,
            ActionFeedbackKind::Check => Self::Dim,
            ActionFeedbackKind::Call => Self::Pulse,
            ActionFeedbackKind::Raise => Self::Bounce,
            ActionFeedbackKind::AllIn => Self::Impact,
        }
    }
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

#[expect(
    clippy::type_complexity,
    reason = "the ParamSet keeps overlapping Bevy feedback queries disjoint"
)]
pub(super) fn animate_texas_action_feedback(
    time: Res<Time>,
    state: Res<TexasChipTableState>,
    mut visuals: ParamSet<(
        Query<(&mut TexasFoldCard, &mut UiTransform, &mut ImageNode)>,
        Query<(&TexasPlayerShake, &mut UiTransform)>,
    )>,
) {
    for (mut card, mut transform, mut image) in &mut visuals.p0() {
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
    for (panel, mut transform) in &mut visuals.p1() {
        let shake_x = state
            .actions
            .get(&panel.0)
            .filter(|label| label.kind == ActionFeedbackKind::AllIn)
            .map_or(0.0, |label| {
                TableStatusEffect::from(label.kind)
                    .visual(label.elapsed)
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
