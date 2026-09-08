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
pub use composition::rebuild_ui;
use connection::{ConnectionScreen, HostGamePicker};
use header::Header;
use lobby::LobbyScreen;
pub use lobby::{LobbyMetrics, LobbySeatSelector};
use settings::SettingsModal;
pub use state::*;
