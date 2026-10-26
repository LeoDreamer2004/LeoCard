use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use leocard_ui::ScrollContentHeight;

pub(crate) fn scroll_modal_content<Marker: Component>(
    mut wheels: MessageReader<MouseWheel>,
    mut scrolls: Query<
        (
            &RelativeCursorPosition,
            &mut ScrollPosition,
            &ComputedNode,
            Option<&ScrollContentHeight>,
        ),
        With<Marker>,
    >,
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
    for (cursor, mut position, node, content_height) in &mut scrolls {
        if cursor.cursor_over() {
            let maximum = content_height.map_or_else(
                || ((node.content_size().y - node.size().y) * node.inverse_scale_factor()).max(0.0),
                |height| height.maximum(node),
            );
            position.y = (position.y - delta).clamp(0.0, maximum);
        }
    }
}
