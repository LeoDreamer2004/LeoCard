use super::TexasSoundAssets;
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Resource)]
pub(crate) struct TexasHoldemAssets {
    pub poker_chips: HashMap<u16, Handle<Image>>,
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
            sounds: TexasSoundAssets::load(asset_server),
        }
    }
}

pub(super) fn load_texas_holdem_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(TexasHoldemAssets::load(&asset_server));
}
