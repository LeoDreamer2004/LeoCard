//! Display order and artwork belong to the gallery; game rules stay in the catalogue.

use crate::app::games::{GameDescriptor, SUPPORTED_GAMES};
use bevy::prelude::Color;

pub(super) struct EntryDesign {
    pub game: GameDescriptor,
    pub image: &'static str,
    pub accent: Color,
}

pub(super) const ENTRY_DESIGNS: [EntryDesign; 5] = [
    EntryDesign {
        game: SUPPORTED_GAMES[4],
        image: "ui/home/entries/mahjong.png",
        accent: Color::srgb(0.40, 0.90, 0.70),
    },
    EntryDesign {
        game: SUPPORTED_GAMES[2],
        image: "ui/home/entries/shengji.png",
        accent: Color::srgb(1.0, 0.52, 0.66),
    },
    EntryDesign {
        game: SUPPORTED_GAMES[1],
        image: "ui/home/entries/texas.png",
        accent: Color::srgb(0.72, 0.58, 1.0),
    },
    EntryDesign {
        game: SUPPORTED_GAMES[3],
        image: "ui/home/entries/uno.png",
        accent: Color::srgb(1.0, 0.80, 0.38),
    },
    EntryDesign {
        game: SUPPORTED_GAMES[0],
        image: "ui/home/entries/qigui.png",
        accent: Color::srgb(0.74, 0.67, 1.0),
    },
];
