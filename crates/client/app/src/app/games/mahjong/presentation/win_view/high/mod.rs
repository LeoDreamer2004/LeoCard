mod assets;
mod material;
mod motion;
mod view;

pub(crate) use assets::MahjongWinShowcaseAssets;
pub(crate) use material::MahjongWinInkMaterial;
pub(crate) use motion::{MAHJONG_INK_SHOWCASE_DURATION, animate_mahjong_win_showcases};
pub(super) use view::render_high_win_showcase;
