//! 顶层连接、房间、设置与牌桌页面。

mod composition;
mod connection;
mod header;
mod lobby;
mod settings;
mod state;

use super::*;
use bevy::prelude::*;
use bevy::ui::{FocusPolicy, RelativeCursorPosition};
pub use composition::render_ui;
use connection::{render_connection, render_host_game_picker};
use header::add_header;
use lobby::render_lobby;
pub use lobby::{connected_lobby_player_count, render_seat_selector};
pub use settings::render_settings_modal;
pub use state::*;
