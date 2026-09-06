//! 德州扑克客户端表现层。
//!
//! 房间、网络、聊天、个人资料与通用控件由 `app` 公共层提供；本模块只包含
//! 德州特有的牌桌、筹码、行动反馈、摊牌和音效编排。

mod action_feedback;
pub mod actions;
mod audio;
mod cards;
mod chips;
mod controls;
mod input;
mod labels;
mod lobby;
mod players;
mod settlement;
mod showdown;
mod state;
mod view;

use super::*;
pub use action_feedback::*;
pub use audio::*;
use bevy::audio::Volume;
use bevy::prelude::*;
use bevy::ui::{FocusPolicy, RelativeCursorPosition};
pub use cards::*;
pub use chips::*;
use controls::*;
pub use input::*;
use labels::*;
pub use lobby::*;
use players::*;
use settlement::*;
pub use showdown::*;
pub use state::*;
use std::collections::{HashMap, HashSet};
pub use view::*;

pub const TEXAS_SHOWDOWN_REVEAL_DURATION: f32 = 2.6;
