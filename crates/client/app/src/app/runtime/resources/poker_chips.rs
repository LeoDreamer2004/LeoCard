use bevy::prelude::*;
use std::collections::HashMap;

pub(crate) const POKER_CHIP_DENOMINATIONS: [u16; 5] = [100, 25, 10, 5, 1];

#[derive(Default)]
pub(crate) struct PokerChipAssets {
    pub squid: Handle<Image>,
    images: HashMap<u16, Handle<Image>>,
}

impl PokerChipAssets {
    pub(crate) fn load(server: &AssetServer) -> Self {
        Self {
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
