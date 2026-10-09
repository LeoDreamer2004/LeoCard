//! 通用客户端输入系统。

mod state;
mod ui;

pub(crate) use state::*;
pub(crate) use ui::*;

mod plugin;
pub(crate) use plugin::InputPlugin;
