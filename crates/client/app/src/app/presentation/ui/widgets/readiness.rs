use super::{add_text, spawn_node};
use crate::app::presentation::UiComposer;
use crate::app::{presentation::READY, runtime::UiAssets};
use bevy::{picking::Pickable, prelude::*};

pub(crate) fn add_ready_badge(
    commands: &mut UiComposer,
    avatar: Entity,
    avatar_size: f32,
    assets: &UiAssets,
) {
    let size = (avatar_size * 0.47).max(13.0);
    let check = spawn_node(
        commands,
        avatar,
        Node {
            position_type: PositionType::Absolute,
            right: px(-3),
            bottom: px(-2),
            width: px(size),
            height: px(size),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            border_radius: BorderRadius::all(percent(50)),
            ..default()
        },
        Some(READY),
    );
    commands.entity(check).insert(Pickable::IGNORE);
    let label = add_text(commands, check, "✓", size * 2.0 / 3.0, Color::WHITE, assets);
    commands.entity(label).insert(Pickable::IGNORE);
}
