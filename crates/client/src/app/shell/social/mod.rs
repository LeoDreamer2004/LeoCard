//! 玩家互动投掷物与通用得分收集演出。

pub mod actions;
mod animation;
mod core;
mod state;

use super::*;
pub use animation::*;
use bevy::audio::Volume;
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use bevy::window::PrimaryWindow;
pub use core::*;
pub use state::*;
use std::collections::HashMap;

pub const INTERACTION_COOLDOWN_MASK_FRAMES: usize = 48;
