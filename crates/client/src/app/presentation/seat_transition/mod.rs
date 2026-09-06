//! 从准备席位到游戏牌桌的座位移动演出。

use bevy::prelude::*;
use leocard_protocol::{MatchId, PlayerId};

pub const START_GAME_SEAT_MOVE_DURATION: f32 = 0.72;

mod state;
mod systems;

pub use state::*;
pub use systems::*;
