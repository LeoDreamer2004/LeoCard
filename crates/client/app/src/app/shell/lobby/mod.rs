//! 通用等待大厅及其操作。

mod actions;
mod entrance;
mod input;
mod plugin;
mod seat_menu;
mod summary_actions;
mod view;

pub(crate) use actions::*;
pub(crate) use entrance::{LobbyEntranceState, LobbySeatedAvatar};
use input::*;
pub(super) use plugin::*;
use seat_menu::handle_lobby_seat_right_click;
pub(crate) use summary_actions::{SummaryActions, add_summary_actions};
pub(crate) use view::*;
