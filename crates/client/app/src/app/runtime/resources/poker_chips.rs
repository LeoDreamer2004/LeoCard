use bevy::prelude::*;
use std::collections::HashMap;

pub(crate) const DEEPSEEK_MARKER_ASSETS: [&str; 8] = [
    "ui/texas_holdem/squid/deepseek/eating.png",
    "ui/texas_holdem/squid/deepseek/holding_sign.png",
    "ui/texas_holdem/squid/deepseek/greeting.png",
    "ui/texas_holdem/squid/deepseek/surprised.png",
    "ui/texas_holdem/squid/deepseek/holding_rice.png",
    "ui/texas_holdem/squid/deepseek/smug.png",
    "ui/texas_holdem/squid/deepseek/standing.png",
    "ui/texas_holdem/squid/deepseek/cheering.png",
];

pub(crate) const POKER_CHIP_DENOMINATIONS: [u16; 5] = [100, 25, 10, 5, 1];

#[derive(Default)]
pub(crate) struct PokerChipAssets {
    pub squid: Handle<Image>,
    pub deepseek: [Handle<Image>; DEEPSEEK_MARKER_ASSETS.len()],
    images: HashMap<u16, Handle<Image>>,
}

impl PokerChipAssets {
    pub(crate) fn load(server: &AssetServer) -> Self {
        Self {
            deepseek: DEEPSEEK_MARKER_ASSETS.map(|path| server.load(path)),
            squid: server.load("ui/texas_holdem/squid_marker.png"),
            images: [
                (1, "WhiteBlue"),
                (5, "RedWhite"),
                (10, "BlueWhite"),
                (25, "GreenWhite"),
                (100, "BlackWhite"),
            ]
            .map(|(value, name)| {
                (
                    value,
                    server.load(format!("vendor/kenney/boardgame/PNG/Chips/chip{name}.png")),
                )
            })
            .into(),
        }
    }

    pub(crate) fn image(&self, denomination: u16) -> Handle<Image> {
        self.images
            .get(&denomination)
            .expect("standard poker chip denomination")
            .clone()
    }
}
