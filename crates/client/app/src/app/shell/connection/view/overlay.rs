use crate::app::shell::UiState;
use bevy::prelude::*;
use leocard_ui::{TransitionVisuals, UiComposer, fade_panel};

#[derive(Component)]
pub(in crate::app::shell::connection) struct HomeIdentityPanel;

pub(in crate::app::shell::connection) fn animate_home_overlays(
    time: Res<Time>,
    mut ui: ResMut<UiState>,
    mut commands: UiComposer,
    mut panels: Query<(Entity, &mut UiTransform), With<HomeIdentityPanel>>,
    children: Query<&Children>,
    mut visuals: TransitionVisuals,
) {
    let home = &mut ui.home;
    let previous = home.identity_progress;
    let direction = if home.identity_open { 1.0 } else { -1.0 };
    home.identity_progress =
        (previous + direction * time.delta_secs().min(0.05) / 0.2).clamp(0.0, 1.0);
    let progress = home.identity_progress;
    let eased = 1.0 - (1.0 - progress).powi(3);
    for (entity, mut transform) in &mut panels {
        transform.translation = Val2::px(0.0, -12.0 * (1.0 - eased));
        transform.scale = Vec2::splat(0.96 + 0.04 * eased);
        fade_panel(
            entity,
            progress * progress * (3.0 - 2.0 * progress),
            &children,
            &mut visuals,
            &mut commands,
        );
    }
    if previous > 0.0 && progress == 0.0 {
        ui.dirty = true;
    }
}
