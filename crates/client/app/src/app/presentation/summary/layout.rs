use bevy::picking::Pickable;
use bevy::prelude::*;
use leocard_ui::{UiComposer, spawn_node};

/// 用独立容器按面板实际高度居中，面板自身的位移仍留给入场动画。
#[track_caller]
pub(crate) fn spawn_centered_summary(
    commands: &mut UiComposer,
    parent: Entity,
    horizontal_inset: Val,
    panel: Node,
) -> Entity {
    let layout = spawn_node(
        commands,
        parent,
        Node {
            position_type: PositionType::Absolute,
            left: horizontal_inset,
            right: horizontal_inset,
            top: px(0),
            bottom: px(0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            ..default()
        },
        None,
    );
    commands.entity(layout).insert(Pickable::IGNORE);
    spawn_node(
        commands,
        layout,
        Node {
            width: percent(100),
            flex_shrink: 0.0,
            ..panel
        },
        None,
    )
}
