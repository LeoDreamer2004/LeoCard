mod audio;
mod common;
mod context;
mod high;
mod major;
mod render;
mod stage;
mod tile_motion;
mod timeline;

pub(crate) use audio::{
    MahjongShowcasePlayback, MahjongShowcaseSounds, play_mahjong_showcase_sounds,
};
pub(crate) use common::*;
use context::WinShowcaseContext;
use high::render_high_win_showcase;
pub(crate) use high::{
    MAHJONG_INK_SHOWCASE_DURATION, MahjongWinInkMaterial, MahjongWinShowcaseAssets,
    animate_mahjong_win_showcases,
};
pub(crate) use major::{
    MAHJONG_MAJOR_SHOWCASE_DURATION, MahjongMajorFanAssets, MahjongWinArtMaterial,
    animate_mahjong_major_showcases, mahjong_major_fan,
};
use major::{MajorShowcaseContext, render_major_win_showcase};
pub(crate) use render::*;
use stage::*;
pub(crate) use tile_motion::animate_mahjong_showcase_tiles;
