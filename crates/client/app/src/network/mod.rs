#[cfg(feature = "web")]
mod browser;
mod client;
mod handshake;
#[cfg(not(feature = "web"))]
mod native;
#[cfg(feature = "web")]
use browser as backend;
#[cfg(not(feature = "web"))]
use native as backend;
#[cfg(not(feature = "web"))]
mod connection;
#[cfg(not(feature = "web"))]
mod lan;
#[cfg(not(feature = "web"))]
mod room_info;
mod state;
#[cfg(test)]
mod tests;
#[cfg(not(feature = "web"))]
mod worker;

pub use client::*;
#[cfg(not(feature = "web"))]
pub use lan::{LanRoom, LanRoomBrowser};
pub use state::*;

#[cfg(feature = "web")]
pub(crate) use browser::RoomInfoQueries;
#[cfg(feature = "web")]
pub use browser::{LanRoom, LanRoomBrowser};
#[cfg(not(feature = "web"))]
pub(crate) use room_info::RoomInfoQueries;

#[cfg(feature = "web")]
pub(crate) use browser::pending_room as pending_web_room;
