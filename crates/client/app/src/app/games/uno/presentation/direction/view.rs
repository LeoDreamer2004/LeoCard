use bevy::picking::Pickable;
use bevy::prelude::*;
use leocard_ui::UiComposer;

#[derive(Component)]
pub(crate) struct UnoDirectionLoop;

pub(crate) fn mount_uno_direction_loop(commands: &mut UiComposer, table: Entity) {
    commands.keyed_child(
        table,
        "direction-loop",
        (
            UnoDirectionLoop,
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                right: px(0),
                top: px(0),
                bottom: px(0),
                ..default()
            },
            Pickable::IGNORE,
        ),
    );
}
