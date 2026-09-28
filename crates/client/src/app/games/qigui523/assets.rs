use bevy::prelude::*;

#[derive(Resource)]
pub(crate) struct QiGui523Assets {
    pub sequence_airplane: Handle<Image>,
    pub action_play: Handle<Image>,
    pub action_play_hover: Handle<Image>,
    pub action_pass: Handle<Image>,
    pub action_pass_hover: Handle<Image>,
    pub action_hint: Handle<Image>,
    pub action_hint_hover: Handle<Image>,
    pub pass_marker: Handle<Image>,
}

impl QiGui523Assets {
    pub(super) fn load(asset_server: &AssetServer) -> Self {
        Self {
            sequence_airplane: asset_server.load("ui/effects/sequence_airplane.png"),
            action_play: asset_server.load("ui/qigui523/play-normal.png"),
            action_play_hover: asset_server.load("ui/qigui523/play-hover.png"),
            action_pass: asset_server.load("ui/qigui523/pass-normal.png"),
            action_pass_hover: asset_server.load("ui/qigui523/pass-hover.png"),
            action_hint: asset_server.load("ui/qigui523/hint-normal.png"),
            action_hint_hover: asset_server.load("ui/qigui523/hint-hover.png"),
            pass_marker: asset_server.load("ui/qigui523/pass-marker.png"),
        }
    }
}

pub(super) fn load_qigui523_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(QiGui523Assets::load(&asset_server));
}
