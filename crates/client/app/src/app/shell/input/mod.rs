//! 通用客户端输入系统。

#[cfg(feature = "android")]
mod android;
#[cfg(feature = "web")]
mod web;

mod back;
mod ui;

pub(crate) use ui::*;

mod plugin;
pub(crate) use plugin::InputPlugin;
