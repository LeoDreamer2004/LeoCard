//! 顶层连接、房间、设置与牌桌页面。

use bevy::prelude::*;
use bevy::ui::{FocusPolicy, RelativeCursorPosition};
use leocard_client::{NetworkState, TcpGameClient};
use leocard_protocol::{GameKind, SeatId, TABLE_SEAT_COUNT, UnoPendingSwapView};
use leocard_shengji::ShengjiRuleSet;
use leocard_uno::UnoRuleSet;

use super::*;

mod composition;
mod connection;
mod header;
mod lobby;
mod settings;
mod state;

pub use composition::render_ui;
use connection::{render_connection, render_host_game_picker};
use header::add_header;
use lobby::render_lobby;

pub use lobby::{connected_lobby_player_count, render_seat_selector};
pub use settings::render_settings_modal;
pub use state::*;
