//! 通用等待大厅及其操作。

mod actions;
mod input;
mod mode;
mod plugin;
mod view;

pub(crate) use actions::*;
use input::*;
pub(crate) use mode::*;
pub(super) use plugin::*;
pub(crate) use view::*;
