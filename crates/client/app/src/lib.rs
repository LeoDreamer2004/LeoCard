//! 桌面、Android 和浏览器共用的客户端应用、状态和平台适配。

#[cfg(not(any(feature = "desktop", feature = "android", feature = "web")))]
compile_error!("请选择 desktop、android 或 web 平台 feature");
#[cfg(any(
    all(feature = "desktop", feature = "android"),
    all(feature = "desktop", feature = "web"),
    all(feature = "android", feature = "web")
))]
compile_error!("平台 feature 只能选择一个，请使用 --no-default-features 切换平台");

extern crate self as leocard_client;

mod app;
mod model;
mod network;
pub mod platform;
mod player;

pub use model::{ActiveGameMeta, ClientModel, ClientPhaseRef};
pub use network::{
    GameClient, LanRoom, LanRoomBrowser, LocalPlayerConnection, NETWORK_ROOM_ID, NetworkStartError,
    NetworkState,
};
pub use platform::{open_url, request_image};
pub use player::*;

/// Launch the shared desktop, Android and browser application.
pub fn run() {
    app::run();
}
