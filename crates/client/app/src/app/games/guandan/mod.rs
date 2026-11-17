//! 掼蛋客户端输入与表现层。

mod actions;
mod arrangement;
mod cards;
mod catalog;
mod controls;
mod counter;
mod drawer;
mod feedback;
mod hand;
mod hand_layout;
mod hit_testing;
mod input;
mod lobby;
mod play;
mod players;
mod plugin;
mod screen;
mod state;
pub(super) mod summary;
mod table;
#[cfg(test)]
mod tests;
mod tribute;
mod voices;

pub(crate) use actions::GuandanUiAction;
pub(crate) use plugin::GuandanPlugin;
pub(super) use screen::GuandanScreen;
