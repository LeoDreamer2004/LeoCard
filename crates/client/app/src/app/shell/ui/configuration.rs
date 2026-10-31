//! Bounded configuration lists share scrolling and preserve their offset during composition.

use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use leocard_ui::{UiComposer, spawn_node};

#[derive(Component)]
pub(super) struct ConfigurationScroll;

#[track_caller]
pub(crate) fn add_configuration_scroll(
    commands: &mut UiComposer,
    parent: Entity,
    key: impl Into<String>,
    gap: f32,
) -> Entity {
    let viewport = commands.keyed_child(
        parent,
        key.into(),
        Node {
            width: percent(100),
            min_height: px(0),
            ..default()
        },
    );
    let scroll = commands.keyed_child(
        viewport,
        "scroll",
        (
            Node {
                min_width: px(0),
                min_height: px(0),
                flex_grow: 1.0,
                flex_basis: px(0),
                flex_direction: FlexDirection::Column,
                overflow: Overflow::scroll_y(),
                ..default()
            },
            ConfigurationScroll,
            RelativeCursorPosition::default(),
        ),
    );
    commands.entity(scroll).init(ScrollPosition::default());
    // Keep rows at their natural height rather than squeezing controls to fit the viewport.
    spawn_node(
        commands,
        scroll,
        Node {
            width: percent(100),
            flex_shrink: 0.0,
            flex_direction: FlexDirection::Column,
            row_gap: px(gap),
            // Tooltips escape visually via OverrideClip without enlarging the scroll range.
            overflow: Overflow::clip(),
            ..default()
        },
        None,
    )
}

pub(super) fn clamp_configuration_scroll(
    mut scrolls: Query<(&ComputedNode, &mut ScrollPosition), With<ConfigurationScroll>>,
) {
    for (node, mut position) in &mut scrolls {
        let maximum =
            ((node.content_size().y - node.size().y) * node.inverse_scale_factor()).max(0.0);
        let clamped = position.y.clamp(0.0, maximum);
        if position.y != clamped {
            position.y = clamped;
        }
    }
}
