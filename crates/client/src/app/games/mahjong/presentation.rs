//! 麻将牌张、吃碰杠、补花、和牌与推牌演出。

mod claim;
mod tiles;
mod win_animation;
mod win_view;

use super::*;
pub use claim::*;
pub use tiles::*;
pub use win_animation::*;
pub use win_view::*;

const MAHJONG_WIN_PUSH_DURATION: f32 = 0.42;
