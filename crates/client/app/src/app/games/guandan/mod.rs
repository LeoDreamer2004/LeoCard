//! 掼蛋客户端输入与表现层。

mod actions;
mod cards;
mod controls;
mod counter;
mod hand;
mod input;
mod lobby;
mod players;
mod plugin;
mod screen;
mod state;
pub(super) mod summary;
mod table;

pub(crate) use actions::GuandanUiAction;
pub(crate) use plugin::GuandanPlugin;
pub(super) use screen::GuandanScreen;
