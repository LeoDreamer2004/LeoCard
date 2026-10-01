use super::{CozyModalBackdrop, CozyModalPanel, ModalAnimations};
use crate::app::presentation::{TransitionVisuals, ease_out_cubic, fade_panel};
use bevy::prelude::*;

const BACKDROP_ALPHA: f32 = 0.76;

pub(crate) fn cozy_backdrop_color(progress: f32) -> Color {
    Color::srgba(
        0.005,
        0.015,
        0.012,
        BACKDROP_ALPHA * ease_out_cubic(progress),
    )
}

pub(crate) fn cozy_panel_transform(progress: f32) -> UiTransform {
    let visible = ease_out_cubic(progress);
    let mut transform = UiTransform::from_translation(Val2::px(0.0, 18.0 * (1.0 - visible)));
    transform.scale = Vec2::splat(0.96 + 0.04 * visible);
    transform
}

pub(crate) fn advance_modal(progress: &mut f32, open: bool, delta: f32) {
    let direction = if open { 1.0 } else { -1.0 };
    *progress = (*progress + direction * delta / 0.22).clamp(0.0, 1.0);
}

pub(crate) fn animate_cozy_modals(
    animations: Res<ModalAnimations>,
    mut commands: Commands,
    mut backdrops: Query<(&CozyModalBackdrop, &mut BackgroundColor)>,
    mut panels: Query<(Entity, &CozyModalPanel, &mut UiTransform)>,
    children: Query<&Children>,
    mut visuals: TransitionVisuals<Without<CozyModalBackdrop>>,
) {
    for (kind, mut backdrop) in &mut backdrops {
        backdrop.0 = cozy_backdrop_color(animations.progress(kind.0));
    }
    for (entity, kind, mut transform) in &mut panels {
        let progress = animations.progress(kind.0);
        *transform = cozy_panel_transform(progress);
        let opacity = progress * progress * (3.0 - 2.0 * progress);
        fade_panel(entity, opacity, &children, &mut visuals, &mut commands);
    }
}
