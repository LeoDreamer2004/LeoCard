use super::layout::MahjongLayout as Layout;
use super::{
    MahjongAssets, MahjongTileHighlight, MahjongTileMaterial, MahjongTileSize, MahjongTileVisual,
    MahjongUiState, add_mahjong_tile_material,
};
use bevy::picking::Pickable;
use bevy::picking::hover::PickingInteraction;
use bevy::prelude::*;
use leocard_mahjong::MahjongTile;
use leocard_ui::UiComposer;

/// A fixed hit region covers both the face-down and upright hand without hover oscillation.
#[derive(Component)]
pub(super) struct MahjongReadyHandRegion;

#[derive(Component)]
pub(super) struct MahjongReadyHandTile {
    region: Entity,
    revealed: bool,
}

pub(super) fn add_mahjong_ready_hand_tile(
    commands: &mut UiComposer,
    hand: Entity,
    tile: MahjongTile,
    index: usize,
    assets: &MahjongAssets,
    materials: &mut Assets<MahjongTileMaterial>,
) {
    let slot = commands.keyed_child(
        hand,
        format!("ready-tile:{tile:?}"),
        (
            Node {
                width: px(Layout::OWN_WIDTH),
                min_width: px(Layout::OWN_WIDTH),
                height: px(Layout::OWN_HEIGHT),
                margin: UiRect::right(px(Layout::OWN_STEP - Layout::OWN_WIDTH)),
                overflow: Overflow::visible(),
                ..default()
            },
            Pickable::IGNORE,
            ZIndex(index as i32),
        ),
    );
    // Share the ordinary upright hand's material and the table's face-down tile geometry.
    for revealed in [false, true] {
        let entity = add_mahjong_tile_material(
            commands,
            slot,
            MahjongTileVisual {
                elevation: 0.0,
                kind: revealed.then_some(tile.kind()),
                size: if revealed {
                    MahjongTileSize::OwnHand
                } else {
                    MahjongTileSize::OwnConcealedMeld
                },
                index,
                highlight: MahjongTileHighlight::None,
                deal: None,
                relative: 0,
            },
            assets,
            materials,
        );
        commands
            .entity(entity)
            .insert((
                MahjongReadyHandTile {
                    region: hand,
                    revealed,
                },
                if revealed {
                    Visibility::Hidden
                } else {
                    Visibility::Inherited
                },
            ))
            .entry::<Node>()
            .and_modify(|mut node| {
                node.position_type = PositionType::Absolute;
                node.left = px(0);
                node.bottom = px(0);
                node.margin = UiRect::ZERO;
            });
    }
}

pub(super) fn sync_mahjong_ready_hand_peek(
    ui: Res<MahjongUiState>,
    regions: Query<&PickingInteraction, With<MahjongReadyHandRegion>>,
    mut tiles: Query<(&MahjongReadyHandTile, &mut Visibility)>,
) {
    for (tile, mut visibility) in &mut tiles {
        let peek = regions.get(tile.region).is_ok_and(|interaction| {
            if cfg!(target_os = "android") {
                ui.ready_hand_peek
            } else {
                matches!(
                    interaction,
                    PickingInteraction::Hovered | PickingInteraction::Pressed
                )
            }
        });
        visibility.set_if_neq(if peek == tile.revealed {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
}
