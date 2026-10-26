use super::state::{MARKER_SIZE, Marker, SquidTableState};
use crate::app::games::texas_holdem::TexasChipTableState;
use crate::app::runtime::UiAssets;
use bevy::picking::Pickable;
use bevy::prelude::*;
use leocard_ui::{UiComposer, spawn_node};

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
                    width: px(MARKER_SIZE),
                    height: px(MARKER_SIZE),
                    ..default()
                },
                marker_transform(marker),
                ImageNode::new(marker_image(marker, assets).clone()),
                SquidMarkerSprite(index),
                Pickable::IGNORE,
            ),
        );
    }
}

pub(crate) fn animate_squid_markers(
    time: Res<Time>,
    assets: Res<UiAssets>,
    mut state: ResMut<TexasChipTableState>,
    mut sprites: Query<(
        &SquidMarkerSprite,
        &mut Node,
        &mut UiTransform,
        &mut ImageNode,
    )>,
) {
    state.squid.tick(time.delta_secs());
    for (sprite, mut node, mut transform, mut image) in &mut sprites {
        let Some(marker) = state.squid.markers.get(sprite.0) else {
            continue;
        };
        let expected = marker_image(marker, &assets);
        if image.image != *expected {
            image.image = expected.clone();
        }
        node.left = px(marker.position.x);
        node.top = percent(marker.position.z * 100.0);
        *transform = marker_transform(marker);
    }
}

fn marker_transform(marker: &Marker) -> UiTransform {
    UiTransform {
        translation: Val2::px(0.0, marker.position.y),
        rotation: Rot2::radians(marker.rotation),
        scale: Vec2::splat(if marker.deepseek.is_some() { 1.2 } else { 1.0 }),
    }
}

fn marker_image<'a>(marker: &Marker, assets: &'a UiAssets) -> &'a Handle<Image> {
    match marker.deepseek {
        Some(index) => &assets.poker_chips.deepseek[index],
        None => &assets.poker_chips.squid,
    }
}
