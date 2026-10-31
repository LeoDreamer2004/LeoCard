//! 房主权威、传输无关的房间会话。
//!
//! Tokio TCP 层只负责把连接映射为 [`ConnectionId`]、解码消息、调用
//! [`HostSession::handle`]，再把 [`Delivery`] 写回指定连接。

mod achievements;
mod automation;
mod blackjack;
mod doudizhu;
mod error;
mod lifecycle;
mod mahjong;
mod mc_guess;
mod player;
mod qigui523;
mod room;
mod session;
mod shengji;
mod texas;
mod uno;

use automation::{AUTOMATIC_ACTION_DELAY, AutomaticActionDelayState};
pub use blackjack::BlackjackSession;
pub use doudizhu::DoudizhuSession;
use ed25519_dalek::{Signature, VerifyingKey};
pub use error::HostError;
use leocard_protocol::{
    AVATAR_DIMENSION, JoinRequest, MAX_AVATAR_BYTES, MatchId, RoomId, ServerMessage,
};
pub use mahjong::MahjongSession;
pub use mc_guess::McGuessSession;
pub use qigui523::QiGui523Session;
pub use room::RoomSession;
pub use session::{GameSetup, HostSession};
pub use shengji::ShengjiSession;
pub use texas::{AdapterError, TablePlayer, TexasAdapter, TexasSession};
pub use uno::UnoSession;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ConnectionId(pub u64);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Delivery {
    pub recipient: ConnectionId,
    pub message: ServerMessage,
}

fn valid_identity_proof(room_id: RoomId, request: &JoinRequest) -> bool {
    let Ok(verifying_key) = VerifyingKey::from_bytes(&request.profile_id.0) else {
        return false;
    };
    let Ok(signature) = Signature::try_from(request.identity_signature.as_slice()) else {
        return false;
    };
    verifying_key
        .verify_strict(&request.identity_payload(room_id), &signature)
        .is_ok()
}

fn new_match_id() -> MatchId {
    let mut bytes = [0; 16];
    bytes[..8].copy_from_slice(&fastrand::u64(..).to_be_bytes());
    bytes[8..].copy_from_slice(&fastrand::u64(..).to_be_bytes());
    MatchId(bytes)
}

fn valid_avatar_png(png: &[u8]) -> bool {
    if png.is_empty() || png.len() > MAX_AVATAR_BYTES {
        return false;
    }
    image::load_from_memory_with_format(png, image::ImageFormat::Png)
        .is_ok_and(|image| image.width() == AVATAR_DIMENSION && image.height() == AVATAR_DIMENSION)
}
