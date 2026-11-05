use crate::app::runtime::TableSurfaceAssets;
use bevy::prelude::*;
use leocard_game_common::mahjong::{
    MahjongDragon, MahjongFlower, MahjongSuit, MahjongTileKind, MahjongWind, build_deck,
};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Resource)]
pub(crate) struct MahjongTileAssets {
    pub tiles: HashMap<MahjongTileKind, Handle<Image>>,
    pub tile_heights: HashMap<MahjongTileKind, Handle<Image>>,
    pub tile_back: Handle<Image>,
    pub table: TableSurfaceAssets,
}

impl MahjongTileAssets {
    pub(crate) fn load(asset_server: &AssetServer) -> Self {
        let kinds = build_deck()
            .into_iter()
            .map(|tile| tile.kind())
            .collect::<HashSet<_>>();
        Self {
            tiles: kinds
                .iter()
                .copied()
                .map(|kind| (kind, asset_server.load(mahjong_tile_asset_path(kind))))
                .collect(),
            tile_heights: kinds
                .into_iter()
                .map(|kind| {
                    (
                        kind,
                        asset_server.load(mahjong_tile_height_asset_path(kind)),
                    )
                })
                .collect(),
            tile_back: asset_server.load("cards/mahjong/hong-kong/back.png"),
            table: TableSurfaceAssets::load(
                asset_server,
                "ui/mahjong",
                Vec4::new(83.0, 83.0, 1172.0, 1164.0) / 1254.0,
                Vec4::new(10.0, 8.0, 1244.0, 1245.0) / 1254.0,
            ),
        }
    }
}

pub(super) fn load_mahjong_tile_assets(mut commands: Commands, server: Res<AssetServer>) {
    commands.insert_resource(MahjongTileAssets::load(&server));
}

pub(crate) fn mahjong_tile_asset_path(kind: MahjongTileKind) -> String {
    let name = match kind {
        MahjongTileKind::Suited {
            suit: MahjongSuit::Characters,
            rank,
        } => format!("characters-{rank}"),
        MahjongTileKind::Suited {
            suit: MahjongSuit::Bamboo,
            rank,
        } => format!("bamboo-{rank}"),
        MahjongTileKind::Suited {
            suit: MahjongSuit::Dots,
            rank,
        } => format!("dots-{rank}"),
        MahjongTileKind::Wind(MahjongWind::East) => "wind-east".to_owned(),
        MahjongTileKind::Wind(MahjongWind::South) => "wind-south".to_owned(),
        MahjongTileKind::Wind(MahjongWind::West) => "wind-west".to_owned(),
        MahjongTileKind::Wind(MahjongWind::North) => "wind-north".to_owned(),
        MahjongTileKind::Dragon(MahjongDragon::Red) => "dragon-red".to_owned(),
        MahjongTileKind::Dragon(MahjongDragon::Green) => "dragon-green".to_owned(),
        MahjongTileKind::Dragon(MahjongDragon::White) => "dragon-white".to_owned(),
        MahjongTileKind::Flower(MahjongFlower::Spring) => "season-spring".to_owned(),
        MahjongTileKind::Flower(MahjongFlower::Summer) => "season-summer".to_owned(),
        MahjongTileKind::Flower(MahjongFlower::Autumn) => "season-autumn".to_owned(),
        MahjongTileKind::Flower(MahjongFlower::Winter) => "season-winter".to_owned(),
        MahjongTileKind::Flower(MahjongFlower::Plum) => "flower-plum".to_owned(),
        MahjongTileKind::Flower(MahjongFlower::Orchid) => "flower-orchid".to_owned(),
        MahjongTileKind::Flower(MahjongFlower::Bamboo) => "flower-bamboo".to_owned(),
        MahjongTileKind::Flower(MahjongFlower::Chrysanthemum) => "flower-chrysanthemum".to_owned(),
    };
    format!("cards/mahjong/hong-kong/{name}.png")
}

pub(crate) fn mahjong_tile_height_asset_path(kind: MahjongTileKind) -> String {
    mahjong_tile_asset_path(kind).replace("/hong-kong/", "/hong-kong-height/")
}
