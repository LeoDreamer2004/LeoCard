//! Tile assets, materials and calibrated table projection shared by mahjong games.
mod assets;
mod geometry;
mod plugin;
mod projection;
mod surfaces;
mod tile;

pub(crate) use assets::MahjongTileAssets;
#[cfg(test)]
pub(crate) use assets::{mahjong_tile_asset_path, mahjong_tile_height_asset_path};
pub(crate) use geometry::MahjongTileGeometry;
pub(crate) use plugin::{MahjongAssetsSet, MahjongPresentationPlugin};
pub(crate) use projection::{
    CAMERA_POSITION, MahjongProjectionFrame, MahjongProjectionViewport, MahjongTableRoot,
    MahjongTileIndicator, REFERENCE_DESIGN_HEIGHT, REFERENCE_SIZE, WORLD_ORIGIN, WORLD_SCALE,
    camera_canvas, ground_from_layout, overlay_tile_scale, own_overlay, project_layout,
    project_logical, project_reference, projected_overlay, reference_ray, world_layout,
};
pub(crate) use tile::*;
