//! UI 动作类型与按功能域拆分的处理系统。

mod connection;
mod lobby;
mod navigation;
mod plugin;
mod types;

use super::*;
use bevy::log::warn;
use bevy::prelude::*;
pub use plugin::{UiActionPlugin, UiActionSet, dispatch_domain_actions};
use types::ButtonInteractions;
pub use types::*;
