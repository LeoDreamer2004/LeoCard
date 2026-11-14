use super::{MahjongTileMaterial, MahjongTileSurface, MahjongUiState};
use bevy::picking::hover::PickingInteraction;
use bevy::prelude::*;

/// The entire concealed row stays pickable while its tiles change their local face visibility.
#[derive(Component)]
pub(super) struct MahjongReadyHandRegion;

#[derive(Component)]
pub(super) struct MahjongReadyHandTile(pub Entity);

pub(super) fn sync_mahjong_ready_hand_peek(
    ui: Res<MahjongUiState>,
    regions: Query<&PickingInteraction, With<MahjongReadyHandRegion>>,
    tiles: Query<(&MahjongReadyHandTile, &MahjongTileSurface)>,
    mut materials: ResMut<Assets<MahjongTileMaterial>>,
) {
    for (tile, surface) in &tiles {
        let peek = regions.get(tile.0).is_ok_and(|interaction| {
            if cfg!(target_os = "android") {
                ui.ready_hand_peek
            } else {
                matches!(
                    interaction,
                    PickingInteraction::Hovered | PickingInteraction::Pressed
                )
            }
        });
        if let Some(mut material) = materials.get_mut(&surface.0) {
            material.params.y = if peek { 0.0 } else { 1.0 };
        }
    }
}
