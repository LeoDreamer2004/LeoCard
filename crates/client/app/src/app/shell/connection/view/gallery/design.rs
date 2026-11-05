//! Display order and artwork belong to the gallery; game rules stay in the catalogue.

use crate::app::games::{GameDescriptor, SUPPORTED_GAMES};
use bevy::prelude::Color;

#[derive(Clone, Copy)]
pub(super) enum CardSurface {
    Clouds,
    Suits,
    Ribbons,
}

pub(super) struct EntryDesign {
    pub game: GameDescriptor,
    pub image: &'static str,
    pub accent: Color,
    pub surface: CardSurface,
}

pub(super) const ENTRY_DESIGNS: [EntryDesign; 10] = [
    EntryDesign {
        game: SUPPORTED_GAMES[4],
        image: "ui/home/entries/mahjong.png",
        surface: CardSurface::Clouds,
        accent: Color::srgb(0.40, 0.90, 0.70),
    },
    EntryDesign {
        game: SUPPORTED_GAMES[2],
        image: "ui/home/entries/shengji.png",
        surface: CardSurface::Suits,
        accent: Color::srgb(1.0, 0.52, 0.66),
    },
    EntryDesign {
        game: SUPPORTED_GAMES[1],
        image: "ui/home/entries/texas.png",
        surface: CardSurface::Suits,
        accent: Color::srgb(0.72, 0.58, 1.0),
    },
    EntryDesign {
        game: SUPPORTED_GAMES[3],
        image: "ui/home/entries/uno.png",
        surface: CardSurface::Ribbons,
        accent: Color::srgb(1.0, 0.80, 0.38),
    },
    EntryDesign {
        game: SUPPORTED_GAMES[0],
        image: "ui/home/entries/qigui.png",
        surface: CardSurface::Suits,
        accent: Color::srgb(0.74, 0.67, 1.0),
    },
    EntryDesign {
        game: SUPPORTED_GAMES[5],
        image: "ui/home/entries/minecraft.png",
        surface: CardSurface::Ribbons,
        accent: Color::srgb(0.45, 0.90, 0.35),
    },
    EntryDesign {
        game: SUPPORTED_GAMES[6],
        image: "ui/home/entries/blackjack.png",
        surface: CardSurface::Suits,
        accent: Color::srgb(0.94, 0.76, 0.31),
    },
    EntryDesign {
        game: SUPPORTED_GAMES[7],
        image: "ui/home/entries/doudizhu.png",
        surface: CardSurface::Suits,
        accent: Color::srgb(1.0, 0.68, 0.30),
    },
    EntryDesign {
        game: SUPPORTED_GAMES[8],
        image: "ui/home/entries/guandan.png",
        surface: CardSurface::Suits,
        accent: Color::srgb(0.95, 0.55, 0.76),
    },
    EntryDesign {
        game: SUPPORTED_GAMES[9],
        image: "ui/home/entries/mahjong.png",
        surface: CardSurface::Clouds,
        accent: Color::srgb(0.96, 0.72, 0.20),
    },
];
