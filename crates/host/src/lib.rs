//! 房主权威、传输无关的房间会话。
//!
//! Tokio TCP 层只负责把连接映射为 [`ConnectionId`]、解码消息、调用
//! [`HostSession::handle`]，再把 [`Delivery`] 写回指定连接。

mod mahjong;
mod qigui523;
mod room;
mod session;
mod shengji;
mod texas_holdem;
mod uno;

use ed25519_dalek::{Signature, VerifyingKey};
use leocard_protocol::{
    AVATAR_DIMENSION, JoinRequest, MAX_AVATAR_BYTES, MatchId, PlayerId, RoomId, ServerMessage,
};
use leocard_qigui523::RuleError;
pub use mahjong::MahjongSession;
pub use qigui523::QiGui523Session;
pub use room::RoomSession;
pub use session::{GameSetup, HostSession};
pub use shengji::ShengjiSession;
use std::fmt;
use std::time::Duration;
pub use texas_holdem::{AdapterError, TablePlayer, TexasHoldemAdapter, TexasHoldemSession};
pub use uno::UnoSession;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ConnectionId(pub u64);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Delivery {
    pub recipient: ConnectionId,
    pub message: ServerMessage,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HostError {
    InvalidRules(RuleError),
    InvalidTexasRules(leocard_texas_holdem::RuleError),
    InvalidShengjiGame(leocard_shengji::GameError),
    InvalidUnoGame(leocard_uno::GameError),
    InvalidMahjongGame(leocard_mahjong::GameError),
    TexasAdapter(AdapterError),
    InvalidDeckSize { expected: usize, actual: usize },
    InvalidDeckContents,
}

impl fmt::Display for HostError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRules(error) => error.fmt(f),
            Self::InvalidTexasRules(error) => error.fmt(f),
            Self::InvalidShengjiGame(error) => error.fmt(f),
            Self::InvalidUnoGame(error) => error.fmt(f),
            Self::InvalidMahjongGame(error) => error.fmt(f),
            Self::TexasAdapter(error) => error.fmt(f),
            Self::InvalidDeckSize { expected, actual } => {
                write!(f, "牌堆张数错误：应为 {expected}，实际为 {actual}")
            }
            Self::InvalidDeckContents => f.write_str("牌堆有缺牌、重复牌或非法牌"),
        }
    }
}

impl std::error::Error for HostError {}

impl From<RuleError> for HostError {
    fn from(value: RuleError) -> Self {
        Self::InvalidRules(value)
    }
}

impl From<leocard_texas_holdem::RuleError> for HostError {
    fn from(value: leocard_texas_holdem::RuleError) -> Self {
        Self::InvalidTexasRules(value)
    }
}

impl From<AdapterError> for HostError {
    fn from(value: AdapterError) -> Self {
        Self::TexasAdapter(value)
    }
}

impl From<leocard_shengji::GameError> for HostError {
    fn from(value: leocard_shengji::GameError) -> Self {
        Self::InvalidShengjiGame(value)
    }
}

impl From<leocard_uno::GameError> for HostError {
    fn from(value: leocard_uno::GameError) -> Self {
        Self::InvalidUnoGame(value)
    }
}

impl From<leocard_mahjong::GameError> for HostError {
    fn from(value: leocard_mahjong::GameError) -> Self {
        Self::InvalidMahjongGame(value)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct TurnTimerState {
    player: PlayerId,
    base_remaining: Duration,
    reserve_remaining: Vec<Duration>,
}

pub(crate) const AUTO_PLAY_DELAY: Duration = Duration::from_secs(1);

#[derive(Clone, Debug)]
pub(crate) struct AutoPlayDelayState {
    player: PlayerId,
    remaining: Duration,
}

pub(crate) fn valid_identity_proof(room_id: RoomId, request: &JoinRequest) -> bool {
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

pub(crate) fn new_match_id() -> MatchId {
    let mut bytes = [0; 16];
    bytes[..8].copy_from_slice(&fastrand::u64(..).to_be_bytes());
    bytes[8..].copy_from_slice(&fastrand::u64(..).to_be_bytes());
    MatchId(bytes)
}

pub(crate) fn valid_avatar_png(png: &[u8]) -> bool {
    if png.is_empty() || png.len() > MAX_AVATAR_BYTES {
        return false;
    }
    image::load_from_memory_with_format(png, image::ImageFormat::Png)
        .is_ok_and(|image| image.width() == AVATAR_DIMENSION && image.height() == AVATAR_DIMENSION)
}
