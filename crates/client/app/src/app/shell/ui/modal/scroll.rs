use super::ModalInputScope;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use leocard_ui::ScrollContentHeight;

type WheelViewport<'a> = (
    Entity,
    &'a RelativeCursorPosition,
    &'a mut ScrollPosition,
    &'a ComputedNode,
    Option<&'a ScrollContentHeight>,
);

pub(crate) fn scroll_modal_content<Marker: Component>(
    mut wheels: MessageReader<MouseWheel>,
    scope: ModalInputScope,
    mut scrolls: Query<WheelViewport<'_>, With<Marker>>,
) {
    let delta = wheels
        .read()
        .map(|wheel| match wheel.unit {
            MouseScrollUnit::Line => wheel.y * 40.0,
            MouseScrollUnit::Pixel => wheel.y,
        })
        .sum::<f32>();
    if delta == 0.0 {
        return;
    }
    for (entity, cursor, mut position, node, content_height) in &mut scrolls {
        if cursor.cursor_over() && scope.allows(entity) {
            let maximum = content_height.map_or_else(
                || ((node.content_size().y - node.size().y) * node.inverse_scale_factor()).max(0.0),
                |height| height.maximum(node),
            );
            position.y = (position.y - delta).clamp(0.0, maximum);
        }
    }
}
