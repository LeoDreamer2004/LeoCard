//! Consistent separators between game-specific groups in settings.
use bevy::prelude::*;
use leocard_ui::{UiComposer, spawn_node};

pub(super) fn add_settings_separator(commands: &mut UiComposer, parent: Entity) {
    spawn_node(
        commands,
        parent,
        Node {
            width: percent(100),
            height: px(1),
            flex_shrink: 0.0,
            margin: UiRect::vertical(px(8)),
            ..default()
        },
        Some(Color::srgba(0.70, 0.68, 0.78, 0.36)),
    );
}
