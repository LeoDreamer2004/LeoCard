//! 从准备席位到游戏牌桌的座位移动演出。

mod state;
mod systems;

use bevy::prelude::*;
pub use state::*;
pub use systems::*;

pub const START_GAME_SEAT_MOVE_DURATION: f32 = 0.72;
