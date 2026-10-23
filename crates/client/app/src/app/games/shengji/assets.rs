use super::ShengjiSoundAssets;
use bevy::prelude::*;
use leocard_shengji::ShengjiSuit;

#[derive(Resource)]
pub(crate) struct ShengjiAssets {
    pub target: Handle<Image>,
    pub dart: Handle<Image>,
    pub bid_capsule: Handle<Image>,
    bid_icons: Handle<Image>,
}

pub(super) fn load_shengji_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(ShengjiAssets::load(&asset_server));
    commands.insert_resource(ShengjiSoundAssets::load(&asset_server));
}

impl ShengjiAssets {
    pub(super) fn load(asset_server: &AssetServer) -> Self {
        Self {
            target: asset_server.load("ui/effects/shengji_target.png"),
            dart: asset_server.load("ui/effects/shengji_dart.png"),
            bid_capsule: asset_server.load("ui/shengji/bid-capsule.png"),
            bid_icons: asset_server.load("ui/shengji/bid-icons.png"),
        }
    }

    pub(super) fn bid_icon(&self, suit: Option<ShengjiSuit>, enabled: bool) -> ImageNode {
        let column = match suit {
            Some(ShengjiSuit::Diamond) => 0,
            Some(ShengjiSuit::Club) => 1,
            Some(ShengjiSuit::Heart) => 2,
            Some(ShengjiSuit::Spade) => 3,
            None => 4,
        };
        // Five 192px cells per row: recessed first, raised second.
        let origin = Vec2::new(column as f32 * 192.0, f32::from(enabled) * 192.0);
        ImageNode {
            image: self.bid_icons.clone(),
            rect: Some(Rect::from_corners(origin, origin + Vec2::splat(192.0))),
            ..default()
        }
    }
}
