use super::state::SquidTableState;
use crate::app::games::texas_holdem::TexasChipTableState;
use crate::app::presentation::{UiComposer, spawn_node};
use crate::app::runtime::UiAssets;
use bevy::picking::Pickable;
use bevy::prelude::*;

#[derive(Component)]
pub(crate) struct SquidMarkerSprite(usize);

pub(crate) fn add_squid_markers(
    commands: &mut UiComposer,
    table: Entity,
    state: &SquidTableState,
    assets: &UiAssets,
) {
    if state.markers.is_empty() {
        return;
    }
    let layer = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            right: px(0),
            top: px(0),
            bottom: px(0),
            ..default()
        },
        None,
    );
    commands
        .entity(layer)
        .insert((ZIndex(46), Pickable::IGNORE));
    for (index, marker) in state.markers.iter().enumerate() {
        commands.child(
            layer,
            (
                Node {
                    position_type: PositionType::Absolute,
                    left: px(marker.position.x),
                    top: percent(marker.position.z * 100.0),
                    width: px(34),
                    height: px(34),
                    ..default()
                },
                UiTransform {
                    translation: Val2::px(0.0, marker.position.y),
                    rotation: Rot2::radians(marker.rotation),
                    ..default()
                },
                ImageNode::new(assets.poker_chips.squid.clone()),
                SquidMarkerSprite(index),
                Pickable::IGNORE,
            ),
        );
    }
}

pub(crate) fn animate_squid_markers(
    time: Res<Time>,
    mut state: ResMut<TexasChipTableState>,
    mut sprites: Query<(&SquidMarkerSprite, &mut Node, &mut UiTransform)>,
) {
    state.squid.tick(time.delta_secs());
    for (sprite, mut node, mut transform) in &mut sprites {
        let Some(marker) = state.squid.markers.get(sprite.0) else {
            continue;
        };
        node.left = px(marker.position.x);
        node.top = percent(marker.position.z * 100.0);
        transform.translation = Val2::px(0.0, marker.position.y);
        transform.rotation = Rot2::radians(marker.rotation);
    }
}
