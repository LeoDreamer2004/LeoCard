use super::ShengjiSoundAssets;
use bevy::prelude::*;

#[derive(Resource)]
pub(crate) struct ShengjiAssets {
    pub target: Handle<Image>,
    pub dart: Handle<Image>,
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
        }
    }
}
