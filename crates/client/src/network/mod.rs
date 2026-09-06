mod worker;

#[cfg(test)]
mod tests;

use std::collections::VecDeque;
use std::fmt;
use std::io;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use leocard_host::HostSession;
use leocard_mahjong::MahjongRuleSet;
use leocard_protocol::{
    ChatMessage, ClientCommand, ClientMessage, MahjongEvent, PlayerGameProfiles, PlayerInteraction,
    ReconnectToken, RoomId, ServerEvent, ServerMessage, ShengjiEvent, TexasHoldemEvent, UnoEvent,
};
use leocard_qigui523::{QiGuiRuleSet, build_deck};
use leocard_shengji::{ShengjiRuleSet, build_deck_for};
use leocard_texas_holdem::TexasHoldemRuleSet;
use leocard_uno::{UnoRuleSet, build_deck_for_rules};
use tokio::sync::mpsc::{Receiver, Sender};

use crate::{ClientModel, LocalPlayerProfile, PlayerIdentity};
use worker::run_network;

/// 当前协议中一个监听端口只承载一个房间，因此客户端无需在地址之外再输入房间号。
pub const NETWORK_ROOM_ID: RoomId = RoomId(523);
const RECONNECT_ATTEMPTS: u8 = 6;
const COMMAND_QUEUE: usize = 64;
const RECONNECT_CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const CONNECTION_HEARTBEAT_TIMEOUT: Duration = Duration::from_secs(12);
const PING_INTERVAL: Duration = Duration::from_secs(3);
const EVENT_QUEUE: usize = 512;

type CommandSender = Sender<ClientMessage>;
type CommandReceiver = Receiver<ClientMessage>;

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

enum NetworkLaunch {
    Host {
        port: u16,
        session: Box<HostSession>,
    },
    Join {
        address: String,
    },
}

enum NetworkEvent {
    Connected(String),
    Reconnecting(String),
    Message(Box<ServerMessage>),
    Failed(String),
}

type EventQueue = Arc<Mutex<VecDeque<NetworkEvent>>>;

#[derive(Clone)]
pub struct LocalPlayerConnection {
    name: String,
    avatar_png: Option<Vec<u8>>,
    identity: PlayerIdentity,
    reference_points: i32,
    completed_games: u32,
    game_profiles: PlayerGameProfiles,
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

    fn temporary(name: &str, avatar_png: Option<Vec<u8>>) -> Result<Self, NetworkStartError> {
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

fn push_event(events: &EventQueue, event: NetworkEvent) {
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

fn normalize_server_address(address: &str) -> Result<String, NetworkStartError> {
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

/// Bevy 主线程使用的非阻塞 TCP 客户端。所有套接字 I/O 都在独立 Tokio 线程中运行。
pub struct TcpGameClient {
    model: ClientModel,
    commands: CommandSender,
    events: EventQueue,
    state: NetworkState,
    _worker: thread::JoinHandle<()>,
}

impl TcpGameClient {
    /// 在所有网卡上监听指定端口，并让房主通过回环 TCP 加入自己的房间。
    pub fn host(name: &str, port: u16, rules: QiGuiRuleSet) -> Result<Self, NetworkStartError> {
        Self::host_with_avatar(name, port, rules, None)
    }

    pub fn host_with_avatar(
        name: &str,
        port: u16,
        rules: QiGuiRuleSet,
        avatar_png: Option<Vec<u8>>,
    ) -> Result<Self, NetworkStartError> {
        Self::host_with_profile(
            port,
            rules,
            LocalPlayerConnection::temporary(name, avatar_png)?,
        )
    }

    pub fn host_with_profile(
        port: u16,
        rules: QiGuiRuleSet,
        player: LocalPlayerConnection,
    ) -> Result<Self, NetworkStartError> {
        if port == 0 {
            return Err(NetworkStartError::InvalidPort);
        }
        let mut deck = build_deck(rules.deck_count);
        fastrand::shuffle(&mut deck);
        let session = HostSession::qigui523(NETWORK_ROOM_ID, port, rules, deck)
            .map_err(|error| NetworkStartError::Worker(io::Error::other(error)))?;
        Self::spawn(
            player,
            NetworkLaunch::Host {
                port,
                session: Box::new(session),
            },
            format!("正在开放 0.0.0.0:{port}"),
        )
    }

    pub fn host_texas_holdem(
        name: &str,
        port: u16,
        rules: TexasHoldemRuleSet,
    ) -> Result<Self, NetworkStartError> {
        Self::host_texas_holdem_with_profile(
            port,
            rules,
            LocalPlayerConnection::temporary(name, None)?,
        )
    }

    pub fn host_texas_holdem_with_profile(
        port: u16,
        rules: TexasHoldemRuleSet,
        player: LocalPlayerConnection,
    ) -> Result<Self, NetworkStartError> {
        if port == 0 {
            return Err(NetworkStartError::InvalidPort);
        }
        let mut deck = leocard_texas_holdem::build_deck(rules.short_deck);
        fastrand::shuffle(&mut deck);
        let session = HostSession::texas_holdem(NETWORK_ROOM_ID, port, rules, deck)
            .map_err(|error| NetworkStartError::Worker(io::Error::other(error)))?;
        Self::spawn(
            player,
            NetworkLaunch::Host {
                port,
                session: Box::new(session),
            },
            format!("正在开放 0.0.0.0:{port}"),
        )
    }

    pub fn host_shengji_with_profile(
        port: u16,
        rules: ShengjiRuleSet,
        player: LocalPlayerConnection,
    ) -> Result<Self, NetworkStartError> {
        if port == 0 {
            return Err(NetworkStartError::InvalidPort);
        }
        let mut deck = build_deck_for(rules.deck_count);
        fastrand::shuffle(&mut deck);
        let session = HostSession::shengji(NETWORK_ROOM_ID, port, rules, deck)
            .map_err(|error| NetworkStartError::Worker(io::Error::other(error)))?;
        Self::spawn(
            player,
            NetworkLaunch::Host {
                port,
                session: Box::new(session),
            },
            format!("正在开放 0.0.0.0:{port}"),
        )
    }

    pub fn host_uno_with_profile(
        port: u16,
        rules: UnoRuleSet,
        player: LocalPlayerConnection,
    ) -> Result<Self, NetworkStartError> {
        if port == 0 {
            return Err(NetworkStartError::InvalidPort);
        }
        let mut deck = build_deck_for_rules(rules);
        fastrand::shuffle(&mut deck);
        let session = HostSession::uno(NETWORK_ROOM_ID, port, rules, deck)
            .map_err(|error| NetworkStartError::Worker(io::Error::other(error)))?;
        Self::spawn(
            player,
            NetworkLaunch::Host {
                port,
                session: Box::new(session),
            },
            format!("正在开放 0.0.0.0:{port}"),
        )
    }

    pub fn host_mahjong_with_profile(
        port: u16,
        rules: MahjongRuleSet,
        player: LocalPlayerConnection,
    ) -> Result<Self, NetworkStartError> {
        if port == 0 {
            return Err(NetworkStartError::InvalidPort);
        }
        let mut deck = leocard_mahjong::build_deck();
        fastrand::shuffle(&mut deck);
        let session = HostSession::mahjong(NETWORK_ROOM_ID, port, rules, deck)
            .map_err(|error| NetworkStartError::Worker(io::Error::other(error)))?;
        Self::spawn(
            player,
            NetworkLaunch::Host {
                port,
                session: Box::new(session),
            },
            format!("正在开放 0.0.0.0:{port}"),
        )
    }

    /// 连接一个形如 `192.168.1.20:52300` 的局域网地址。
    pub fn join(name: &str, address: &str) -> Result<Self, NetworkStartError> {
        Self::join_with_avatar(name, address, None)
    }

    pub fn join_with_avatar(
        name: &str,
        address: &str,
        avatar_png: Option<Vec<u8>>,
    ) -> Result<Self, NetworkStartError> {
        Self::join_with_profile(address, LocalPlayerConnection::temporary(name, avatar_png)?)
    }

    pub fn join_with_profile(
        address: &str,
        player: LocalPlayerConnection,
    ) -> Result<Self, NetworkStartError> {
        let address = normalize_server_address(address)?;
        let connecting = format!("正在连接 {address}");
        Self::spawn(player, NetworkLaunch::Join { address }, connecting)
    }

    fn spawn(
        player: LocalPlayerConnection,
        launch: NetworkLaunch,
        connecting: String,
    ) -> Result<Self, NetworkStartError> {
        let (command_tx, command_rx) = tokio::sync::mpsc::channel(COMMAND_QUEUE);
        let events = Arc::new(Mutex::new(VecDeque::with_capacity(EVENT_QUEUE)));
        let mut model = ClientModel::new(NETWORK_ROOM_ID);
        let reconnect_token = ReconnectToken(loop {
            let token = getrandom::u64().map_err(|error| {
                NetworkStartError::Worker(io::Error::other(format!("无法生成重连令牌：{error}")))
            })?;
            if token != 0 {
                break token;
            }
        });
        let join = model.command(player.identity.join_command(
            NETWORK_ROOM_ID,
            player.name.trim(),
            reconnect_token,
            player.reference_points,
            player.completed_games,
            player.game_profiles,
        ));
        let avatar = player
            .avatar_png
            .map(|png| model.command(ClientCommand::SetAvatar { png }));
        let reconnect_join = join.clone();
        let worker_events = Arc::clone(&events);
        let worker = thread::Builder::new()
            .name("leocard-network".to_owned())
            .spawn(move || {
                let runtime = match tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                {
                    Ok(runtime) => runtime,
                    Err(error) => {
                        push_event(
                            &worker_events,
                            NetworkEvent::Failed(format!("无法启动网络运行时：{error}")),
                        );
                        return;
                    }
                };
                if let Err(error) = runtime.block_on(run_network(
                    launch,
                    command_rx,
                    &worker_events,
                    reconnect_join,
                )) {
                    push_event(&worker_events, NetworkEvent::Failed(error));
                }
            })
            .map_err(NetworkStartError::Worker)?;

        let client = Self {
            model,
            commands: command_tx,
            events,
            state: NetworkState::Connecting(connecting),
            _worker: worker,
        };
        let _ = client.commands.try_send(join);
        if let Some(avatar) = avatar {
            let _ = client.commands.try_send(avatar);
        }
        Ok(client)
    }

    pub fn model(&self) -> &ClientModel {
        &self.model
    }

    pub fn state(&self) -> &NetworkState {
        &self.state
    }

    pub fn send(&mut self, command: ClientCommand) -> bool {
        if !matches!(self.state, NetworkState::Connected(_)) {
            return false;
        }
        let message = self.model.command(command);
        self.commands.try_send(message).is_ok()
    }

    pub fn take_player_interactions(&mut self) -> Vec<PlayerInteraction> {
        self.model.take_player_interactions()
    }

    pub fn take_chat_messages(&mut self) -> Vec<ChatMessage> {
        self.model.take_chat_messages()
    }

    pub fn take_texas_holdem_events(&mut self) -> Vec<TexasHoldemEvent> {
        self.model.take_texas_holdem_events()
    }

    pub fn take_shengji_events(&mut self) -> Vec<ShengjiEvent> {
        self.model.take_shengji_events()
    }

    pub fn take_uno_events(&mut self) -> Vec<UnoEvent> {
        self.model.take_uno_events()
    }

    pub fn take_mahjong_events(&mut self) -> Vec<MahjongEvent> {
        self.model.take_mahjong_events()
    }

    /// 把后台线程已经收到的消息应用到模型；返回是否有可见状态变化。
    pub fn poll(&mut self) -> bool {
        let mut changed = false;
        let pending = {
            let mut events = self
                .events
                .lock()
                .expect("network event queue mutex poisoned");
            events.drain(..).collect::<Vec<_>>()
        };
        for event in pending {
            match event {
                NetworkEvent::Connected(address) => {
                    let state = NetworkState::Connected(address);
                    changed |= self.state != state;
                    self.state = state;
                }
                NetworkEvent::Reconnecting(message) => {
                    let state = NetworkState::Reconnecting(message);
                    changed |= self.state != state;
                    self.state = state;
                }
                NetworkEvent::Message(message) => {
                    let visible = !matches!(
                        &message.event,
                        ServerEvent::Heartbeat
                            | ServerEvent::PlayerInteraction(_)
                            | ServerEvent::ChatMessage(_)
                    );
                    changed |= self.model.apply(*message) && visible;
                }
                NetworkEvent::Failed(error) => {
                    let state = NetworkState::Failed(error);
                    changed |= self.state != state;
                    self.state = state;
                }
            }
        }
        changed
    }
}
