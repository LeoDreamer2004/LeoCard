//! Horizontal archive browsing by wheel or pointer drag, including touch mouse input.

use super::ProfileGameContent;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;

#[derive(Component)]
pub(super) struct ProfileArchiveEdge(pub Entity);

pub(super) fn scroll_archive(
    mut wheels: MessageReader<MouseWheel>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut drag: Local<Option<(Entity, f32)>>,
    mut scrolls: Query<
        (
            Entity,
            &RelativeCursorPosition,
            &ComputedNode,
            &mut ScrollPosition,
        ),
        With<ProfileGameContent>,
    >,
) {
    let delta = wheels
        .read()
        .map(|wheel| {
            let delta = if wheel.x.abs() > wheel.y.abs() {
                wheel.x
            } else {
                wheel.y
            };
            -delta
                * if wheel.unit == MouseScrollUnit::Line {
                    40.0
                } else {
                    1.0
                }
        })
        .sum::<f32>();
    if !mouse.pressed(MouseButton::Left) {
        *drag = None;
    }
    for (entity, cursor, node, mut position) in &mut scrolls {
        let maximum =
            ((node.content_size().x - node.size().x) * node.inverse_scale_factor()).max(0.0);
        let mut offset = position.x;
        if cursor.cursor_over() {
            offset += delta;
        }
        if let Some(point) = cursor.normalized {
            let x = point.x * node.size().x * node.inverse_scale_factor();
            if mouse.just_pressed(MouseButton::Left) && cursor.cursor_over() {
                *drag = Some((entity, x));
            }
            if let Some((owner, previous)) = *drag
                && owner == entity
            {
                offset += previous - x;
                *drag = Some((entity, x));
            }
        }
        offset = offset.clamp(0.0, maximum);
        if position.x != offset {
            position.x = offset;
        }
    }
    if drag.is_some_and(|(entity, _)| scrolls.get(entity).is_err()) {
        *drag = None;
    }
}

pub(super) fn update_archive_edge(
    scrolls: Query<(&ComputedNode, &ScrollPosition), With<ProfileGameContent>>,
    mut edges: Query<(&ProfileArchiveEdge, &mut Visibility)>,
) {
    for (edge, mut visible) in &mut edges {
        let more = scrolls.get(edge.0).is_ok_and(|(node, position)| {
            let maximum = (node.content_size().x - node.size().x) * node.inverse_scale_factor();
            maximum - position.x > 1.0
        });
        visible.set_if_neq(if more {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
}
