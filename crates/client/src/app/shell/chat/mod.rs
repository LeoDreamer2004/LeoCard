//! 聊天界面、消息同步与气泡演出。

pub mod actions;
mod presentation;
mod state;
mod view;

use super::*;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::ui::{FocusPolicy, RelativeCursorPosition};
pub use presentation::*;
pub use state::*;
use std::collections::{HashMap, VecDeque};
pub use view::*;

pub const CHAT_PANEL_WIDTH: f32 = 350.0;
/// 将面板本体移出右侧，同时保留其左侧的 32px 折叠箭头。
pub const CHAT_PANEL_HIDDEN_OFFSET: f32 = CHAT_PANEL_WIDTH + 2.0;
