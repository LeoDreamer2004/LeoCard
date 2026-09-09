use bevy::prelude::*;
use leocard_mahjong::{MahjongTileKind, build_deck};
use std::collections::{HashMap, HashSet};

#[derive(Resource)]
pub(crate) struct MahjongAssets {
    pub tiles: HashMap<MahjongTileKind, Handle<Image>>,
    pub tile_heights: HashMap<MahjongTileKind, Handle<Image>>,
    pub tile_back: Handle<Image>,
    pub turn_arrow: Handle<Image>,
}

impl MahjongAssets {
    pub(super) fn load(asset_server: &AssetServer) -> Self {
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
            turn_arrow: asset_server.load("vendor/kenney/ui/PNG/Yellow/Default/arrow_basic_e.png"),
        }
    }
}

pub(super) fn load_mahjong_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(MahjongAssets::load(&asset_server));
}
