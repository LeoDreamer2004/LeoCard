use super::{MahjongStatusImages, create_mahjong_status_images};
use bevy::prelude::*;
use leocard_mahjong::{MahjongTileKind, build_deck};
use std::collections::{HashMap, HashSet};

#[derive(Resource)]
pub(crate) struct MahjongAssets {
    pub tiles: HashMap<MahjongTileKind, Handle<Image>>,
    pub tile_heights: HashMap<MahjongTileKind, Handle<Image>>,
    pub tile_back: Handle<Image>,
    pub(super) status: MahjongStatusImages,
}

impl MahjongAssets {
    pub(super) fn load(asset_server: &AssetServer, images: &mut Assets<Image>) -> Self {
        let kinds = build_deck()
            .into_iter()
            .map(|tile| tile.kind())
            .collect::<HashSet<_>>();
        let tiles = kinds
            .iter()
            .copied()
            .map(|kind| {
                (
                    kind,
                    asset_server.load(crate::app::mahjong_tile_asset_path(kind)),
                )
            })
            .collect();
        let tile_heights = kinds
            .into_iter()
            .map(|kind| {
                (
                    kind,
                    asset_server.load(crate::app::mahjong_tile_height_asset_path(kind)),
                )
            })
            .collect();
        Self {
            tiles,
            tile_heights,
            tile_back: asset_server.load("cards/mahjong/hong-kong/back.png"),
            status: create_mahjong_status_images(images),
        }
    }
}

pub(super) fn load_mahjong_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
) {
    commands.insert_resource(MahjongAssets::load(&asset_server, &mut images));
}
