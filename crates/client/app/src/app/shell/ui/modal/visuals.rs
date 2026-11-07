use super::{CozyModalBackdrop, CozyModalKind, CozyModalPanel, ModalAnimations, ModalDismiss};
use crate::app::presentation::ease_out_cubic;
use crate::app::shell::{ShellOverlayLayer, UiAction};
use bevy::picking::Pickable;
use bevy::prelude::*;
use leocard_ui::{TransitionVisuals, UiComposer, fade_panel};

const BACKDROP_ALPHA: f32 = 0.76;

#[track_caller]
pub(crate) fn add_centered_modal(
    commands: &mut UiComposer,
    parent: Entity,
    layer: ShellOverlayLayer,
    kind: CozyModalKind,
    progress: f32,
    dismiss: Option<UiAction>,
) -> Entity {
    let overlay = commands.child(
        parent,
        (
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                right: px(0),
                top: px(0),
                bottom: px(0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(cozy_backdrop_color(progress)),
            layer.z_index(),
            Pickable::default(),
            CozyModalBackdrop(kind),
        ),
    );
    if let Some(action) = dismiss {
        commands.entity(overlay).insert(ModalDismiss(action));
    }
    overlay
}

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
    mut commands: UiComposer,
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
