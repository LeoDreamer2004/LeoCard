//! 麻将牌张、吃碰杠、补花、和牌与推牌演出。

use super::*;

const MAHJONG_WIN_PUSH_DURATION: f32 = 0.42;

mod claim;
mod tiles;
mod win_animation;
mod win_view;

pub use claim::*;
pub use tiles::*;
pub use win_animation::*;
pub use win_view::*;
