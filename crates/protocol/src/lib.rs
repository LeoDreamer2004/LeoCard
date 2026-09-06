//! 与传输方式无关的联机协议。
//!
//! TCP 层只需读取 4 字节大端长度，再读取对应的 Postcard 负载；业务层始终处理
//! [`ClientMessage`] 和 [`ServerMessage`]。

pub const PROTOCOL_VERSION: u16 = 35;
pub const MAX_FRAME_PAYLOAD: usize = 1024 * 1024;
pub const MAX_PLAYER_NAME_CHARS: usize = 7;
pub const AVATAR_DIMENSION: u32 = 64;
pub const MAX_AVATAR_BYTES: usize = 32 * 1024;
pub const MAX_CHAT_MESSAGE_CHARS: usize = 120;
pub const QUICK_VOICE_COUNT: u8 = 23;
pub const TABLE_SEAT_COUNT: u8 = 6;

pub mod events;
pub mod framing;
pub mod game;
pub mod identity;
pub mod mahjong;
pub mod messages;
pub mod qigui523;
pub mod shengji;
pub mod texas_holdem;
pub mod uno;
pub mod violations;

pub use events::*;
pub use framing::*;
pub use game::*;
pub use identity::*;
pub use mahjong::*;
pub use messages::*;
pub use qigui523::*;
pub use shengji::*;
pub use texas_holdem::*;
pub use uno::*;
pub use violations::*;

#[cfg(test)]
mod tests;
