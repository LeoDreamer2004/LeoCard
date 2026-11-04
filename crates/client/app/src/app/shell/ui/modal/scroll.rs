use super::ModalWheelBlocker;
use bevy::ecs::system::SystemParam;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::ui::{ComputedStackIndex, RelativeCursorPosition};
use leocard_ui::ScrollContentHeight;

#[derive(SystemParam)]
pub(crate) struct ModalWheelScope<'w, 's> {
    blockers: Query<
        'w,
        's,
        (
            Entity,
            &'static ComputedStackIndex,
            &'static InheritedVisibility,
        ),
        With<ModalWheelBlocker>,
    >,
    parents: Query<'w, 's, &'static ChildOf>,
}

impl ModalWheelScope<'_, '_> {
    pub(crate) fn allows(&self, target: Entity) -> bool {
        // Raw wheel messages reach every reader, regardless of picking.
        let top = self
            .blockers
            .iter()
            .filter(|(_, _, visibility)| visibility.get())
            .max_by_key(|(_, order, _)| order.0);
        top.is_none_or(|(modal, _, _)| {
            modal == target
                || self
                    .parents
                    .iter_ancestors(target)
                    .any(|entity| entity == modal)
        })
    }
}

type WheelViewport<'a> = (
    Entity,
    &'a RelativeCursorPosition,
    &'a mut ScrollPosition,
    &'a ComputedNode,
    Option<&'a ScrollContentHeight>,
);

pub(crate) fn scroll_modal_content<Marker: Component>(
    mut wheels: MessageReader<MouseWheel>,
    scope: ModalWheelScope,
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
