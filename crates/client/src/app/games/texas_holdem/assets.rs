use super::TexasSoundAssets;
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Resource)]
pub(crate) struct TexasHoldemAssets {
    pub poker_chips: HashMap<u16, Handle<Image>>,
    pub action_primary: Handle<Image>,
    pub action_primary_hover: Handle<Image>,
    pub action_secondary: Handle<Image>,
    pub action_secondary_hover: Handle<Image>,
    pub action_warning: Handle<Image>,
    pub action_warning_hover: Handle<Image>,
    pub action_pass: Handle<Image>,
    pub action_pass_hover: Handle<Image>,
    pub adjust_left: Handle<Image>,
    pub adjust_left_hover: Handle<Image>,
    pub adjust_right: Handle<Image>,
    pub adjust_right_hover: Handle<Image>,
    pub sounds: TexasSoundAssets,
}

impl TexasHoldemAssets {
    pub(super) fn load(asset_server: &AssetServer) -> Self {
        Self {
            poker_chips: HashMap::from([
                (
                    1,
                    asset_server.load("vendor/kenney/boardgame/PNG/Chips/chipWhiteBlue.png"),
                ),
                (
                    5,
                    asset_server.load("vendor/kenney/boardgame/PNG/Chips/chipRedWhite.png"),
                ),
                (
                    10,
                    asset_server.load("vendor/kenney/boardgame/PNG/Chips/chipBlueWhite.png"),
                ),
                (
                    25,
                    asset_server.load("vendor/kenney/boardgame/PNG/Chips/chipGreenWhite.png"),
                ),
                (
                    100,
                    asset_server.load("vendor/kenney/boardgame/PNG/Chips/chipBlackWhite.png"),
                ),
            ]),
            action_primary: asset_server.load("ui/qigui523/play-normal.png"),
            action_primary_hover: asset_server.load("ui/qigui523/play-hover.png"),
            action_secondary: asset_server.load("ui/qigui523/hint-normal.png"),
            action_secondary_hover: asset_server.load("ui/qigui523/hint-hover.png"),
            action_warning: asset_server.load("ui/qigui523/warning-normal.png"),
            action_warning_hover: asset_server.load("ui/qigui523/warning-hover.png"),
            action_pass: asset_server.load("ui/qigui523/pass-normal.png"),
            action_pass_hover: asset_server.load("ui/qigui523/pass-hover.png"),
            adjust_left: asset_server.load("ui/texas_holdem/adjust-left-normal.png"),
            adjust_left_hover: asset_server.load("ui/texas_holdem/adjust-left-hover.png"),
            adjust_right: asset_server.load("ui/texas_holdem/adjust-right-normal.png"),
            adjust_right_hover: asset_server.load("ui/texas_holdem/adjust-right-hover.png"),
            sounds: TexasSoundAssets::load(asset_server),
        }
    }
}

pub(super) fn load_texas_holdem_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(TexasHoldemAssets::load(&asset_server));
}
