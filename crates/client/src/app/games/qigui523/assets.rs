use bevy::prelude::*;

#[derive(Resource)]
pub(crate) struct QiGui523Assets {
    pub sequence_airplane: Handle<Image>,
}

impl QiGui523Assets {
    pub(super) fn load(asset_server: &AssetServer) -> Self {
        Self {
            sequence_airplane: asset_server.load("ui/effects/sequence_airplane.png"),
        }
    }
}

pub(super) fn load_qigui523_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(QiGui523Assets::load(&asset_server));
}
