//! UI 动作类型与按功能域拆分的处理系统。

mod connection;
mod dispatcher;
mod lobby;
mod navigation;
mod types;

use super::*;
use bevy::log::warn;
use bevy::prelude::*;
pub use dispatcher::handle_buttons;
use types::ButtonInteractions;
pub use types::{LocalUiResources, UiAction};
