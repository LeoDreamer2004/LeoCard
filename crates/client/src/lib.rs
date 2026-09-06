//! 客户端状态与 TCP 传输适配器。状态模型严格只处理协议消息。

mod model;
mod network;
mod player;

pub use model::{ClientModel, ScoreCaptureEffect, ShengjiScoreCaptureEffect};
pub use network::{
    LocalPlayerConnection, NETWORK_ROOM_ID, NetworkStartError, NetworkState, TcpGameClient,
};
pub use player::*;
