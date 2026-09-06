//! 跨游戏共用的终局结算表现。

mod state;
mod systems;

use super::*;
use bevy::prelude::*;
pub use state::*;
pub use systems::*;

pub const SUMMARY_MODAL_ENTRY_DURATION: f32 = 0.55;
pub const SUMMARY_HAND_REVEAL_DURATION: f32 = 1.5;
pub const SUMMARY_ROW_START_DELAY: f32 = 0.38;
pub const SUMMARY_ROW_INTERVAL: f32 = 0.18;
pub const SUMMARY_ROW_ENTRY_DURATION: f32 = 0.32;
pub const SUMMARY_SCORE_COUNT_DURATION: f32 = 0.72;
pub const SUMMARY_ACTIONS_EXTRA_DELAY: f32 = 0.30;
pub const TEXAS_UNCONTESTED_REVEAL_DURATION: f32 = 0.8;
