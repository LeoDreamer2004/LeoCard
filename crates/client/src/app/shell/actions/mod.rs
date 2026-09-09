//! UI 动作类型与按功能域拆分的处理系统。

mod connection;
mod lobby;
mod navigation;
mod plugin;
mod types;

pub(crate) use connection::*;
pub(crate) use lobby::*;
pub(crate) use navigation::*;
pub(crate) use plugin::*;
pub(crate) use types::*;
