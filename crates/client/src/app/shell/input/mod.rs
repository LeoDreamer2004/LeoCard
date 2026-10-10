//! 通用客户端输入系统。

#[cfg(target_os = "android")]
mod android;

mod state;
mod ui;

pub(crate) use state::*;
pub(crate) use ui::*;

mod plugin;
pub(crate) use plugin::InputPlugin;
