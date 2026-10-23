//! 桌面与 Android 共用的客户端应用、状态和平台适配。

extern crate self as leocard_client;

mod app;
mod model;
mod network;
pub mod platform;
mod player;

pub use model::{
    ActiveGameMeta, ClientModel, ClientPhaseRef, ScoreCaptureEffect, ShengjiScoreCaptureEffect,
};
pub use network::{
    LanRoom, LanRoomBrowser, LocalPlayerConnection, NETWORK_ROOM_ID, NetworkStartError,
    NetworkState, TcpGameClient,
};
pub use platform::{open_url, pick_image};
pub use player::*;

/// Launch the shared desktop and Android application.
pub fn run() {
    app::run();
}
