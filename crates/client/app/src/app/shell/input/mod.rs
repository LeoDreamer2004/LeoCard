//! 通用客户端输入系统。

#[cfg(target_os = "android")]
mod android;

mod back;
mod ui;

pub(crate) use ui::*;

mod plugin;
pub(crate) use plugin::InputPlugin;
