use super::super::spawn_node;
use crate::app::presentation::UiComposer;
use bevy::prelude::*;

pub(crate) fn add_avatar_status_overlay(
    commands: &mut UiComposer,
    avatar: Entity,
    size: f32,
    border_color: Color,
) -> Entity {
    let overlay = spawn_node(
        commands,
        avatar,
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            right: px(0),
            top: px(0),
            bottom: px(0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            border: UiRect::all(px(2)),
            border_radius: BorderRadius::all(px(size * 0.2)),
            ..default()
        },
        Some(Color::BLACK.with_alpha(0.72)),
    );
    commands.entity(overlay).insert((
        BorderColor::all(border_color.with_alpha(0.96)),
        Pickable::IGNORE,
        ZIndex(80),
    ));
    overlay
}
