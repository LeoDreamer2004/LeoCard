//! 双升（升级）四人牌桌、亮主、埋底、出牌与闲家计分界面。

pub mod actions;
mod hints;
mod lobby;
mod presentation;
mod settlement;
mod snapshot;
mod state;
mod throw_feedback;
mod view;

use super::*;
use bevy::audio::Volume;
use bevy::prelude::*;
use bevy::ui::{FocusPolicy, RelativeCursorPosition};
pub use hints::*;
pub use lobby::*;
pub use presentation::*;
pub use settlement::*;
pub use snapshot::*;
pub use state::*;
use std::collections::HashSet;
pub use throw_feedback::*;
pub use view::*;

const SHENGJI_SETTLEMENT_MODAL_DELAY: f32 = 3.38;
const SHENGJI_SETTLEMENT_ROW_INTERVAL: f32 = 0.18;
