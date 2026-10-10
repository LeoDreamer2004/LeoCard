//! 客户端状态与 TCP 传输适配器。状态模型严格只处理协议消息。

#[cfg(target_os = "android")]
extern crate self as leocard_client;

#[cfg(target_os = "android")]
mod app;
mod model;
mod network;
pub mod platform;
mod player;
#[cfg(target_os = "android")]
mod updater;

pub use model::{
    ActiveGameMeta, ClientModel, ClientPhaseRef, ScoreCaptureEffect, ShengjiScoreCaptureEffect,
};
pub use network::{
    LocalPlayerConnection, NETWORK_ROOM_ID, NetworkStartError, NetworkState, TcpGameClient,
};
pub use platform::{open_url, pick_image};
pub use player::*;

#[cfg(target_os = "android")]
#[bevy::prelude::bevy_main]
fn main() {
    app::run();
}
