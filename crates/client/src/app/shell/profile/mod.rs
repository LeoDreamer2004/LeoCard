//! 玩家档案界面与各游戏长期统计。

mod actions;

mod animation;
mod plugin;
mod rating;
mod state;
mod stats;
mod view;

pub(crate) use animation::*;
pub(crate) use plugin::ProfilePlugin;
pub(crate) use rating::*;
pub(crate) use state::*;
pub(crate) use stats::*;
pub(crate) use view::*;

pub(crate) use actions::ProfileUiAction;

mod entry;
pub(crate) use entry::ProfileEntry;
