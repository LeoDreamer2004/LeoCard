//! 掼蛋客户端输入与表现层。

mod actions;
mod cards;
mod catalog;
mod controls;
mod counter;
mod feedback;
mod hand;
mod input;
mod lobby;
mod play;
mod players;
mod plugin;
mod screen;
mod state;
pub(super) mod summary;
mod table;
mod tribute;
mod voices;

pub(crate) use actions::GuandanUiAction;
pub(crate) use plugin::GuandanPlugin;
pub(super) use screen::GuandanScreen;
