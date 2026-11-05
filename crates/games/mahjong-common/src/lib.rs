//! Mahjong tile identities and physical decks shared by independent games.
mod tile;

pub use tile::{
    MahjongDragon, MahjongFlower, MahjongSuit, MahjongTile, MahjongTileKind, MahjongWind,
    build_deck,
};
