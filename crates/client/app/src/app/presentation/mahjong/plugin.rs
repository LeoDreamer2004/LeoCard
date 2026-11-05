use super::assets::load_mahjong_tile_assets;
use super::projection::{
    sync_mahjong_projection_frames, sync_mahjong_projection_viewport, sync_mahjong_tile_projection,
};
use super::surfaces::sync_mahjong_tile_surfaces;
use super::tile::sync_mahjong_tile_orientation;
use super::{MahjongProjectionViewport, MahjongTileMaterial};
use bevy::camera::visibility::VisibilitySystems;
use bevy::prelude::*;
use bevy::ui::UiSystems;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct MahjongAssetsSet;

pub(crate) struct MahjongPresentationPlugin;

impl Plugin for MahjongPresentationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(UiMaterialPlugin::<MahjongTileMaterial>::default())
            .init_resource::<MahjongProjectionViewport>()
            .add_systems(Startup, load_mahjong_tile_assets.in_set(MahjongAssetsSet))
            .add_systems(
                PostUpdate,
                (
                    sync_mahjong_projection_viewport,
                    sync_mahjong_tile_orientation,
                    sync_mahjong_tile_projection,
                    sync_mahjong_tile_surfaces,
                    sync_mahjong_projection_frames,
                )
                    .chain()
                    .after(UiSystems::Layout)
                    .before(UiSystems::PostLayout)
                    .before(UiSystems::Stack)
                    .before(VisibilitySystems::VisibilityPropagate),
            );
    }
}
