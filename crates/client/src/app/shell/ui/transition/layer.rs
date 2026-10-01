//! Full-screen layers used by page and game transitions.

use bevy::prelude::*;
use bevy::ui::FocusPolicy;

pub(super) fn add_screen_layer(
    commands: &mut Commands,
    root: Entity,
    z_index: i32,
    marker: impl Component,
) -> Entity {
    let layer = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                right: px(0),
                top: px(0),
                bottom: px(0),
                ..default()
            },
            GlobalZIndex(z_index),
            FocusPolicy::Block,
            marker,
        ))
        .id();
    commands.entity(root).add_child(layer);
    layer
}

pub(super) fn shade_color(alpha: f32) -> BackgroundColor {
    BackgroundColor(Color::srgba(0.025, 0.022, 0.045, alpha))
}
