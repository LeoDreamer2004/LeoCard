mod assets;
mod catalog;
mod impact;
mod material;
mod motion;
mod sequence;
mod view;

pub(crate) use assets::{MahjongMajorFanAssets, mahjong_major_fan};
pub(crate) use material::MahjongWinArtMaterial;
pub(crate) use motion::animate_mahjong_major_showcases;
pub(crate) use sequence::MAHJONG_MAJOR_SHOWCASE_DURATION;
pub(super) use sequence::MajorShowcaseSequence;
pub(super) use view::{MajorShowcaseContext, render_major_win_showcase};
