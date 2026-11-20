//! TCP、WebSocket 和可选 TLS 接入共用同一个权威房间。

mod client;
mod codec;
mod discovery;
mod error;
mod server;
#[cfg(test)]
mod tests;

pub use client::{TcpClient, TcpClientReader, TcpClientWriter};
pub use codec::{read_message, write_message};
pub use discovery::{LanDiscoverySocket, LanRoomAnnouncement, query_room_info};
pub use error::TcpError;
pub use server::{ServerTls, TcpServerHandle};
use std::time::Duration;

const FRAME_HEADER_TIMEOUT: Duration = Duration::from_secs(15);
const FRAME_BODY_TIMEOUT: Duration = Duration::from_secs(10);
const WRITE_TIMEOUT: Duration = Duration::from_secs(5);
