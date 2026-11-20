use crate::{LocalPlayerProfile, PlayerIdentity};
#[cfg(not(feature = "web"))]
use leocard_host::HostSession;
use leocard_platform::time::Duration;
#[cfg(not(feature = "web"))]
use leocard_protocol::ClientMessage;
use leocard_protocol::{
    AvatarFrameId, PlayerGameProfiles, PlayerGender, RoomId, RoomInfo, ServerMessage,
};
#[cfg(not(feature = "web"))]
use leocard_tcp::ServerTls;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::{fmt, io};
#[cfg(not(feature = "web"))]
use tokio::sync::mpsc::{Receiver, Sender};

pub(super) const RECONNECT_ATTEMPTS: u8 = 6;
#[cfg(not(feature = "web"))]
pub(super) const COMMAND_QUEUE: usize = 64;
pub(super) const RECONNECT_CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
pub(super) const CONNECTION_HEARTBEAT_TIMEOUT: Duration = Duration::from_secs(12);
pub(super) const PING_INTERVAL: Duration = Duration::from_secs(3);
pub(super) const EVENT_QUEUE: usize = 512;

/// 当前协议中一个监听端口只承载一个房间，因此客户端无需在地址之外再输入房间号。
pub const NETWORK_ROOM_ID: RoomId = RoomId(523);

#[cfg(not(feature = "web"))]
pub(super) type CommandSender = Sender<ClientMessage>;
#[cfg(feature = "web")]
pub(super) type CommandSender = super::browser::CommandSender;
#[cfg(not(feature = "web"))]
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
    Unsupported(&'static str),
}

impl fmt::Display for NetworkStartError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unsupported(message) => f.write_str(message),
            Self::InvalidPort => f.write_str("端口必须在 1..=65535 范围内"),
            Self::InvalidAddress(address) => {
                write!(
                    f,
                    "{}，当前输入：{address}",
                    if cfg!(feature = "web") {
                        "连接地址需要完整的 ws:// 或 wss:// 地址"
                    } else {
                        "连接地址必须是 主机名或IP:端口"
                    }
                )
            }
            Self::Worker(error) => write!(f, "无法启动网络线程：{error}"),
        }
    }
}

impl std::error::Error for NetworkStartError {}

pub(super) enum NetworkLaunch {
    #[cfg(not(feature = "web"))]
    Host {
        port: u16,
        session: Box<HostSession>,
        tls: Option<ServerTls>,
    },
    Join {
        address: String,
    },
}

pub(super) enum NetworkEvent {
    #[cfg(feature = "web")]
    Resync,
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
        gender: PlayerGender,
        avatar_frame: AvatarFrameId,
    ) -> Self {
        let mut game_profiles = profile.game_profiles().clone();
        game_profiles.gender = gender;
        game_profiles.avatar_frame = avatar_frame;
        Self {
            name: name.into(),
            avatar_png,
            identity: profile.identity.clone(),
            reference_points: profile.reference_points(),
            completed_games: profile.completed_games(),
            game_profiles,
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
    if queue
        .iter()
        .any(|event| matches!(event, NetworkEvent::Failed(_)))
    {
        return;
    }
    if queue.len() >= EVENT_QUEUE {
        queue.clear();
        queue.push_back(NetworkEvent::Failed(
            "客户端处理网络消息过慢，请重新加入房间以同步完整状态".to_owned(),
        ));
        return;
    }
    queue.push_back(event);
}

pub(crate) fn normalize_server_address(address: &str) -> Result<String, NetworkStartError> {
    #[cfg(feature = "web")]
    return super::browser::normalize_address(address);
    #[cfg(not(feature = "web"))]
    normalize_tcp_address(address)
}

#[cfg(not(feature = "web"))]
fn normalize_tcp_address(address: &str) -> Result<String, NetworkStartError> {
    let address = address.trim();
    let invalid = || NetworkStartError::InvalidAddress(address.to_owned());
    let (host, port) = address.rsplit_once(':').ok_or_else(invalid)?;
    let port = port.parse::<u16>().map_err(|_| invalid())?;
    if port == 0 || host.is_empty() {
        return Err(invalid());
    }
    // 含冒号的 IPv6 地址必须使用标准的 `[地址]:端口` 形式，域名和 IPv4 可直接使用。
    if host.contains(':') && !(host.starts_with('[') && host.ends_with(']')) {
        return Err(invalid());
    }
    if host.starts_with('[') != host.ends_with(']') {
        return Err(invalid());
    }
    Ok(address.to_owned())
}

pub(crate) struct RoomInfoResult {
    pub address: String,
    pub info: Option<(RoomInfo, u32)>,
}
