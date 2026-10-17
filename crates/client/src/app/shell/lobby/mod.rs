//! 通用等待大厅及其操作。

mod actions;
mod input;
mod mode;
mod plugin;
mod summary_actions;
mod view;

pub(crate) use actions::*;
use input::*;
pub(crate) use mode::*;
pub(super) use plugin::*;
pub(crate) use summary_actions::{SummaryActions, add_summary_actions};
pub(crate) use view::*;
