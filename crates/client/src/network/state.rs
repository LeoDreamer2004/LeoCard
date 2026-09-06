use crate::{LocalPlayerProfile, PlayerIdentity};
use leocard_host::HostSession;
use leocard_protocol::{ClientMessage, PlayerGameProfiles, RoomId, ServerMessage};
use std::collections::VecDeque;
use std::fmt;
use std::io;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::mpsc::{Receiver, Sender};

pub(super) const RECONNECT_ATTEMPTS: u8 = 6;
pub(super) const COMMAND_QUEUE: usize = 64;
pub(super) const RECONNECT_CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
pub(super) const CONNECTION_HEARTBEAT_TIMEOUT: Duration = Duration::from_secs(12);
pub(super) const PING_INTERVAL: Duration = Duration::from_secs(3);
const EVENT_QUEUE: usize = 512;

/// 当前协议中一个监听端口只承载一个房间，因此客户端无需在地址之外再输入房间号。
pub const NETWORK_ROOM_ID: RoomId = RoomId(523);

pub(super) type CommandSender = Sender<ClientMessage>;
pub(super) type CommandReceiver = Receiver<ClientMessage>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NetworkState {
    Connecting(String),
    Reconnecting(String),
    Connected(String),
    Failed(String),
}

#[derive(Debug)]
pub enum NetworkStartError {
    InvalidPort,
    InvalidAddress(String),
    Worker(io::Error),
}

impl fmt::Display for NetworkStartError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPort => f.write_str("端口必须在 1..=65535 范围内"),
            Self::InvalidAddress(address) => {
                write!(f, "连接地址必须是 主机名或IP:端口，当前输入：{address}")
            }
            Self::Worker(error) => write!(f, "无法启动网络线程：{error}"),
        }
    }
}

impl std::error::Error for NetworkStartError {}

pub(super) enum NetworkLaunch {
    Host {
        port: u16,
        session: Box<HostSession>,
    },
    Join {
        address: String,
    },
}

pub(super) enum NetworkEvent {
    Connected(String),
    Reconnecting(String),
    Message(Box<ServerMessage>),
    Failed(String),
}

pub(super) type EventQueue = Arc<Mutex<VecDeque<NetworkEvent>>>;

#[derive(Clone)]
pub struct LocalPlayerConnection {
    pub(super) name: String,
    pub(super) avatar_png: Option<Vec<u8>>,
    pub(super) identity: PlayerIdentity,
    pub(super) reference_points: i32,
    pub(super) completed_games: u32,
    pub(super) game_profiles: PlayerGameProfiles,
}

impl LocalPlayerConnection {
    pub fn from_profile(
        name: impl Into<String>,
        avatar_png: Option<Vec<u8>>,
        profile: &LocalPlayerProfile,
    ) -> Self {
        Self {
            name: name.into(),
            avatar_png,
            identity: profile.identity.clone(),
            reference_points: profile.reference_points(),
            completed_games: profile.completed_games(),
            game_profiles: profile.game_profiles().clone(),
        }
    }

    pub(super) fn temporary(
        name: &str,
        avatar_png: Option<Vec<u8>>,
    ) -> Result<Self, NetworkStartError> {
        Ok(Self {
            name: name.to_owned(),
            avatar_png,
            identity: PlayerIdentity::generate().map_err(NetworkStartError::Worker)?,
            reference_points: 0,
            completed_games: 0,
            game_profiles: PlayerGameProfiles::default(),
        })
    }
}

pub(super) fn push_event(events: &EventQueue, event: NetworkEvent) {
    let mut queue = events.lock().expect("network event queue mutex poisoned");
    if queue.len() >= EVENT_QUEUE {
        if let Some(index) = queue
            .iter()
            .position(|queued| matches!(queued, NetworkEvent::Message(_)))
        {
            queue.remove(index);
        } else {
            queue.pop_front();
        }
    }
    queue.push_back(event);
}

pub(super) fn normalize_server_address(address: &str) -> Result<String, NetworkStartError> {
    let address = address.trim();
    let invalid = || NetworkStartError::InvalidAddress(address.to_owned());
    let (host, port) = address.rsplit_once(':').ok_or_else(&invalid)?;
    let port = port.parse::<u16>().map_err(|_| invalid())?;
    if port == 0 || host.is_empty() {
        return Err(invalid());
    }
    // 含冒号的 IPv6 地址必须使用标准的 `[地址]:端口` 形式；域名和 IPv4 可直接使用。
    if host.contains(':') && !(host.starts_with('[') && host.ends_with(']')) {
        return Err(invalid());
    }
    if host.starts_with('[') != host.ends_with(']') {
        return Err(invalid());
    }
    Ok(address.to_owned())
}
