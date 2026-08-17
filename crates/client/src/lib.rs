//! 客户端状态以及内存/TCP 传输适配器。状态模型严格只处理协议消息。

use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use ed25519_dalek::{Signer, SigningKey};
use leocard_host::{ConnectionId, HostError, HostSession};
use leocard_protocol::{
    AvatarId, ChatMessage, ClientCommand, ClientMessage, GameCommand, GameEvent, GamePhaseView,
    GameRules, GameSnapshot as AnyGameSnapshot, LobbySnapshot, MatchId, PROTOCOL_VERSION,
    PlayerGameProfiles, PlayerId, PlayerInteraction, PlayerReferenceChange, ProfileId, PublicPlay,
    PublicPlayRecord, QiGui523Command, QiGui523Event, QiGui523Snapshot as GameSnapshot,
    ReconnectToken, RejectReason, RequestId, Revision, RoomId, SeatId, ServerEvent, ServerMessage,
    ShengjiEvent, ShengjiPhaseView, ShengjiSnapshot, TexasHoldemEvent, TexasHoldemPhaseView,
    TexasHoldemSnapshot, TrickView, UnoEvent, UnoPhaseView, UnoSnapshot, join_identity_payload,
};
use leocard_qigui523::{Card, RuleSet, build_deck};
use leocard_shengji::{RuleSet as ShengjiRuleSet, build_deck_for as build_shengji_deck_for};
use leocard_tcp::{TcpClient, TcpServerHandle};
use leocard_texas_holdem::{RuleSet as TexasHoldemRuleSet, build_deck as build_texas_holdem_deck};
use leocard_uno::{RuleSet as UnoRuleSet, build_deck as build_uno_deck};
use tokio::sync::mpsc::{Receiver, Sender};

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

#[derive(Clone)]
pub struct PlayerIdentity {
    signing_key: SigningKey,
}

impl PlayerIdentity {
    pub fn generate() -> io::Result<Self> {
        let mut secret = [0; 32];
        getrandom::fill(&mut secret)
            .map_err(|error| io::Error::other(format!("无法生成玩家身份：{error}")))?;
        Ok(Self::from_secret_bytes(secret))
    }

    pub fn from_secret_bytes(secret: [u8; 32]) -> Self {
        Self {
            signing_key: SigningKey::from_bytes(&secret),
        }
    }

    pub fn secret_bytes(&self) -> [u8; 32] {
        self.signing_key.to_bytes()
    }

    pub fn profile_id(&self) -> ProfileId {
        ProfileId(self.signing_key.verifying_key().to_bytes())
    }

    fn join_command(
        &self,
        room_id: RoomId,
        name: &str,
        reconnect_token: ReconnectToken,
        reference_points: i32,
        completed_games: u32,
        game_profiles: PlayerGameProfiles,
    ) -> ClientCommand {
        let payload = join_identity_payload(
            room_id,
            reconnect_token,
            name,
            reference_points,
            completed_games,
            &game_profiles,
        );
        ClientCommand::Join {
            name: name.to_owned(),
            reconnect_token,
            profile_id: self.profile_id(),
            reference_points,
            completed_games,
            game_profiles,
            identity_signature: self.signing_key.sign(&payload).to_bytes().to_vec(),
        }
    }
}

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

use std::io;

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
    pub fn host(name: &str, port: u16, rules: RuleSet) -> Result<Self, NetworkStartError> {
        Self::host_with_avatar(name, port, rules, None)
    }

    pub fn host_with_avatar(
        name: &str,
        port: u16,
        rules: RuleSet,
        avatar_png: Option<Vec<u8>>,
    ) -> Result<Self, NetworkStartError> {
        let identity = PlayerIdentity::generate().map_err(NetworkStartError::Worker)?;
        Self::host_with_profile(
            name,
            port,
            rules,
            avatar_png,
            identity,
            0,
            0,
            PlayerGameProfiles::default(),
        )
    }

    pub fn host_with_profile(
        name: &str,
        port: u16,
        rules: RuleSet,
        avatar_png: Option<Vec<u8>>,
        identity: PlayerIdentity,
        reference_points: i32,
        completed_games: u32,
        game_profiles: PlayerGameProfiles,
    ) -> Result<Self, NetworkStartError> {
        if port == 0 {
            return Err(NetworkStartError::InvalidPort);
        }
        let mut deck = build_deck(rules.deck_count);
        fastrand::shuffle(&mut deck);
        let session = HostSession::qigui523(NETWORK_ROOM_ID, port, rules, deck)
            .map_err(|error| NetworkStartError::Worker(io::Error::other(error)))?;
        Self::spawn(
            name,
            avatar_png,
            identity,
            reference_points,
            completed_games,
            game_profiles,
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
        let identity = PlayerIdentity::generate().map_err(NetworkStartError::Worker)?;
        Self::host_texas_holdem_with_profile(
            name,
            port,
            rules,
            None,
            identity,
            0,
            0,
            PlayerGameProfiles::default(),
        )
    }

    pub fn host_texas_holdem_with_profile(
        name: &str,
        port: u16,
        rules: TexasHoldemRuleSet,
        avatar_png: Option<Vec<u8>>,
        identity: PlayerIdentity,
        reference_points: i32,
        completed_games: u32,
        game_profiles: PlayerGameProfiles,
    ) -> Result<Self, NetworkStartError> {
        if port == 0 {
            return Err(NetworkStartError::InvalidPort);
        }
        let mut deck = build_texas_holdem_deck(rules.short_deck);
        fastrand::shuffle(&mut deck);
        let session = HostSession::texas_holdem(NETWORK_ROOM_ID, port, rules, deck)
            .map_err(|error| NetworkStartError::Worker(io::Error::other(error)))?;
        Self::spawn(
            name,
            avatar_png,
            identity,
            reference_points,
            completed_games,
            game_profiles,
            NetworkLaunch::Host {
                port,
                session: Box::new(session),
            },
            format!("正在开放 0.0.0.0:{port}"),
        )
    }

    pub fn host_shengji_with_profile(
        name: &str,
        port: u16,
        rules: ShengjiRuleSet,
        avatar_png: Option<Vec<u8>>,
        identity: PlayerIdentity,
        reference_points: i32,
        completed_games: u32,
        game_profiles: PlayerGameProfiles,
    ) -> Result<Self, NetworkStartError> {
        if port == 0 {
            return Err(NetworkStartError::InvalidPort);
        }
        let mut deck = build_shengji_deck_for(rules.deck_count);
        fastrand::shuffle(&mut deck);
        let session = HostSession::shengji(NETWORK_ROOM_ID, port, rules, deck)
            .map_err(|error| NetworkStartError::Worker(io::Error::other(error)))?;
        Self::spawn(
            name,
            avatar_png,
            identity,
            reference_points,
            completed_games,
            game_profiles,
            NetworkLaunch::Host {
                port,
                session: Box::new(session),
            },
            format!("正在开放 0.0.0.0:{port}"),
        )
    }

    pub fn host_uno_with_profile(
        name: &str,
        port: u16,
        rules: UnoRuleSet,
        avatar_png: Option<Vec<u8>>,
        identity: PlayerIdentity,
        reference_points: i32,
        completed_games: u32,
        game_profiles: PlayerGameProfiles,
    ) -> Result<Self, NetworkStartError> {
        if port == 0 {
            return Err(NetworkStartError::InvalidPort);
        }
        let mut deck = build_uno_deck();
        fastrand::shuffle(&mut deck);
        let session = HostSession::uno(NETWORK_ROOM_ID, port, rules, deck)
            .map_err(|error| NetworkStartError::Worker(io::Error::other(error)))?;
        Self::spawn(
            name,
            avatar_png,
            identity,
            reference_points,
            completed_games,
            game_profiles,
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
        let identity = PlayerIdentity::generate().map_err(NetworkStartError::Worker)?;
        Self::join_with_profile(
            name,
            address,
            avatar_png,
            identity,
            0,
            0,
            PlayerGameProfiles::default(),
        )
    }

    pub fn join_with_profile(
        name: &str,
        address: &str,
        avatar_png: Option<Vec<u8>>,
        identity: PlayerIdentity,
        reference_points: i32,
        completed_games: u32,
        game_profiles: PlayerGameProfiles,
    ) -> Result<Self, NetworkStartError> {
        let address = normalize_server_address(address)?;
        let connecting = format!("正在连接 {address}");
        Self::spawn(
            name,
            avatar_png,
            identity,
            reference_points,
            completed_games,
            game_profiles,
            NetworkLaunch::Join { address },
            connecting,
        )
    }

    fn spawn(
        name: &str,
        avatar_png: Option<Vec<u8>>,
        identity: PlayerIdentity,
        reference_points: i32,
        completed_games: u32,
        game_profiles: PlayerGameProfiles,
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
        let join = model.command(identity.join_command(
            NETWORK_ROOM_ID,
            name.trim(),
            reconnect_token,
            reference_points,
            completed_games,
            game_profiles,
        ));
        let avatar = avatar_png.map(|png| model.command(ClientCommand::SetAvatar { png }));
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

async fn run_network(
    launch: NetworkLaunch,
    mut commands: CommandReceiver,
    events: &EventQueue,
    reconnect_join: ClientMessage,
) -> Result<(), String> {
    match launch {
        NetworkLaunch::Host { port, session } => {
            let bind_address = SocketAddr::new(IpAddr::from([0, 0, 0, 0]), port);
            let server = TcpServerHandle::bind(bind_address, *session)
                .await
                .map_err(|error| format!("无法开放端口 {port}：{error}"))?;
            let local_address = SocketAddr::new(IpAddr::from([127, 0, 0, 1]), port);
            let client = TcpClient::connect(local_address)
                .await
                .map_err(|error| format!("房主无法回连本机端口 {port}：{error}"))?;
            let result = run_connected(
                client,
                &mut commands,
                events,
                format!("已监听端口 {port}；请分享 房主局域网IP:{port}"),
            )
            .await;
            let shutdown = server
                .shutdown()
                .await
                .map_err(|error| format!("关闭房间失败：{error}"));
            result.and(shutdown)
        }
        NetworkLaunch::Join { address } => {
            let client = TcpClient::connect(address.as_str())
                .await
                .map_err(|error| format!("连接 {address} 失败：{error}"))?;
            run_joined_network(client, address, &mut commands, events, &reconnect_join).await
        }
    }
}

async fn run_joined_network(
    mut client: TcpClient,
    address: String,
    commands: &mut CommandReceiver,
    events: &EventQueue,
    reconnect_join: &ClientMessage,
) -> Result<(), String> {
    loop {
        let disconnected =
            match run_connected(client, commands, events, format!("已连接 {address}")).await {
                Ok(()) => return Ok(()),
                Err(error) => error,
            };
        drain_pending_commands(commands);

        let mut last_error = disconnected;
        let mut reconnected = None;
        for attempt in 1..=RECONNECT_ATTEMPTS {
            let delay = reconnect_delay(attempt);
            let status = format!(
                "连接已断开；{} 秒后进行第 {attempt}/{RECONNECT_ATTEMPTS} 次重连",
                delay.as_secs()
            );
            push_event(events, NetworkEvent::Reconnecting(status));
            tokio::time::sleep(delay).await;

            match tokio::time::timeout(
                RECONNECT_CONNECT_TIMEOUT,
                TcpClient::connect(address.as_str()),
            )
            .await
            {
                Ok(Ok(replacement)) => {
                    match complete_reconnect_handshake(replacement, reconnect_join, events).await {
                        Ok(replacement) => {
                            reconnected = Some(replacement);
                            break;
                        }
                        Err(error) => {
                            last_error = format!("第 {attempt} 次重连握手失败：{error}");
                        }
                    }
                }
                Ok(Err(error)) => {
                    last_error = format!("第 {attempt} 次重连失败：{error}");
                }
                Err(_) => {
                    last_error = format!("第 {attempt} 次重连超时");
                }
            }
        }

        let Some(replacement) = reconnected else {
            return Err(format!(
                "已连续重连 {RECONNECT_ATTEMPTS} 次，仍无法连接房主：{last_error}"
            ));
        };
        drain_pending_commands(commands);
        client = replacement;
    }
}

async fn complete_reconnect_handshake(
    mut client: TcpClient,
    reconnect_join: &ClientMessage,
    events: &EventQueue,
) -> Result<TcpClient, String> {
    client
        .send(reconnect_join)
        .await
        .map_err(|error| error.to_string())?;
    let message = tokio::time::timeout(RECONNECT_CONNECT_TIMEOUT, client.receive())
        .await
        .map_err(|_| "等待房主确认超时".to_owned())?
        .map_err(|error| error.to_string())?;
    match &message.event {
        ServerEvent::Joined { .. } => {
            push_event(events, NetworkEvent::Message(Box::new(message)));
            Ok(client)
        }
        ServerEvent::Rejected { reason } => Err(format!("房主拒绝恢复身份：{reason:?}")),
        _ => Err("房主未确认恢复玩家身份".to_owned()),
    }
}

async fn run_connected(
    client: TcpClient,
    commands: &mut CommandReceiver,
    events: &EventQueue,
    connected_label: String,
) -> Result<(), String> {
    let (mut reader, mut writer) = client.into_split();
    push_event(events, NetworkEvent::Connected(connected_label));
    let heartbeat_timeout = tokio::time::sleep(CONNECTION_HEARTBEAT_TIMEOUT);
    tokio::pin!(heartbeat_timeout);
    let mut ping_interval = tokio::time::interval(PING_INTERVAL);
    ping_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        tokio::select! {
            _ = &mut heartbeat_timeout => {
                return Err("与房主的连接心跳超时".to_owned());
            }
            message = reader.receive() => {
                let message = message.map_err(|error| format!("与房间的连接已断开：{error}"))?;
                heartbeat_timeout.as_mut().reset(
                    tokio::time::Instant::now() + CONNECTION_HEARTBEAT_TIMEOUT
                );
                if matches!(&message.event, ServerEvent::Heartbeat) {
                    continue;
                }
                let connection_finished = matches!(
                    &message.event,
                    ServerEvent::RoomClosed | ServerEvent::LeftRoom
                );
                push_event(events, NetworkEvent::Message(Box::new(message)));
                if connection_finished {
                    return Ok(());
                }
            }
            command = commands.recv() => {
                let Some(command) = command else { return Ok(()) };
                writer.send(&command).await
                    .map_err(|error| format!("发送游戏指令失败：{error}"))?;
            }
            _ = ping_interval.tick() => {
                let ping = ClientMessage::new(
                    NETWORK_ROOM_ID,
                    RequestId(0),
                    ClientCommand::Ping,
                );
                writer.send(&ping).await
                    .map_err(|error| format!("发送连接心跳失败：{error}"))?;
            }
        }
    }
}

fn drain_pending_commands(commands: &mut CommandReceiver) {
    while commands.try_recv().is_ok() {}
}

fn reconnect_delay(attempt: u8) -> Duration {
    match attempt {
        1 => Duration::ZERO,
        2 => Duration::from_secs(1),
        3 => Duration::from_secs(2),
        4 => Duration::from_secs(4),
        5 => Duration::from_secs(8),
        _ => Duration::from_secs(15),
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScoreCaptureEffect {
    pub player: PlayerId,
    pub cards: Vec<Card>,
    /// 每张牌在终局时所属的玩家；`None` 表示牌来自桌面中央。
    pub source_players: Vec<Option<PlayerId>>,
    pub score_before: u32,
    pub score_after: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShengjiScoreCaptureEffect {
    pub cards: Vec<leocard_shengji::Card>,
    pub score_before: u32,
    pub score_after: u32,
}

#[derive(Clone, Debug)]
pub struct ClientModel {
    room_id: RoomId,
    host_port: Option<u16>,
    next_request: u64,
    latest_revision: Revision,
    you: Option<PlayerId>,
    lobby: Option<LobbySnapshot>,
    game: Option<AnyGameSnapshot>,
    rules: Option<GameRules>,
    avatars: HashMap<AvatarId, Vec<u8>>,
    captured_score_cards: HashMap<PlayerId, Vec<Card>>,
    shengji_collected_score_cards: Vec<leocard_shengji::Card>,
    shengji_pending_trick_score_cards: Vec<leocard_shengji::Card>,
    shengji_finished_event_pending_snapshot: bool,
    last_shengji_score_capture: Option<ShengjiScoreCaptureEffect>,
    shengji_score_capture_serial: u64,
    observed_trick: Option<TrickView>,
    active_match_id: Option<MatchId>,
    room_closed: bool,
    left_room: bool,
    last_rejection: Option<RejectReason>,
    rejection_serial: u64,
    last_notice: Option<String>,
    notice_serial: u64,
    last_play_effect: Option<(PlayerId, PublicPlay)>,
    play_effect_serial: u64,
    last_score_capture: Option<ScoreCaptureEffect>,
    score_capture_serial: u64,
    pending_player_interactions: VecDeque<PlayerInteraction>,
    pending_chat_messages: VecDeque<ChatMessage>,
    pending_texas_holdem_events: VecDeque<TexasHoldemEvent>,
    pending_shengji_events: VecDeque<ShengjiEvent>,
    pending_uno_events: VecDeque<UnoEvent>,
    last_finished_match: Option<(MatchId, Vec<PlayerReferenceChange>)>,
}

impl ClientModel {
    pub fn new(room_id: RoomId) -> Self {
        Self {
            room_id,
            host_port: None,
            next_request: 1,
            latest_revision: Revision(0),
            you: None,
            lobby: None,
            game: None,
            rules: None,
            avatars: HashMap::new(),
            captured_score_cards: HashMap::new(),
            shengji_collected_score_cards: Vec::new(),
            shengji_pending_trick_score_cards: Vec::new(),
            shengji_finished_event_pending_snapshot: false,
            last_shengji_score_capture: None,
            shengji_score_capture_serial: 0,
            observed_trick: None,
            active_match_id: None,
            room_closed: false,
            left_room: false,
            last_rejection: None,
            rejection_serial: 0,
            last_notice: None,
            notice_serial: 0,
            last_play_effect: None,
            play_effect_serial: 0,
            last_score_capture: None,
            score_capture_serial: 0,
            pending_player_interactions: VecDeque::new(),
            pending_chat_messages: VecDeque::new(),
            pending_texas_holdem_events: VecDeque::new(),
            pending_shengji_events: VecDeque::new(),
            pending_uno_events: VecDeque::new(),
            last_finished_match: None,
        }
    }

    pub fn command(&mut self, command: ClientCommand) -> ClientMessage {
        let request_id = RequestId(self.next_request);
        self.next_request += 1;
        ClientMessage::new(self.room_id, request_id, command)
    }

    /// 返回消息是否被接受。错误房间、错误版本和旧修订号均被忽略。
    pub fn apply(&mut self, message: ServerMessage) -> bool {
        if message.protocol_version != PROTOCOL_VERSION
            || message.room_id != self.room_id
            || message.revision < self.latest_revision
        {
            return false;
        }
        self.latest_revision = message.revision;
        match message.event {
            ServerEvent::Joined { you } => {
                self.you = Some(you);
                self.last_rejection = None;
            }
            ServerEvent::AvatarData { id, png } => {
                self.avatars.entry(id).or_insert(png);
            }
            ServerEvent::LobbySnapshot(snapshot) => {
                self.host_port = Some(snapshot.host_port);
                self.room_closed = false;
                self.captured_score_cards.clear();
                self.shengji_collected_score_cards.clear();
                self.shengji_pending_trick_score_cards.clear();
                self.shengji_finished_event_pending_snapshot = false;
                self.last_shengji_score_capture = None;
                self.shengji_score_capture_serial = 0;
                self.observed_trick = None;
                self.active_match_id = None;
                self.last_score_capture = None;
                self.score_capture_serial = 0;
                self.pending_player_interactions.clear();
                self.pending_chat_messages.clear();
                self.pending_texas_holdem_events.clear();
                self.pending_shengji_events.clear();
                self.pending_uno_events.clear();
                self.rules = Some(snapshot.rules.clone());
                self.lobby = Some(snapshot);
                self.game = None;
                self.last_rejection = None;
            }
            ServerEvent::GameSnapshot(AnyGameSnapshot::QiGui523(snapshot)) => {
                self.host_port = Some(snapshot.host_port);
                if self.active_match_id != Some(snapshot.match_id) {
                    self.captured_score_cards.clear();
                    self.observed_trick = None;
                    self.active_match_id = Some(snapshot.match_id);
                    self.last_score_capture = None;
                    self.score_capture_serial = 0;
                    self.pending_player_interactions.clear();
                }
                if let GamePhaseView::Finished {
                    match_id,
                    reference_changes,
                    ..
                } = &snapshot.phase
                {
                    self.last_finished_match = Some((*match_id, reference_changes.clone()));
                }
                self.observe_score_cards(&snapshot);
                self.you = Some(snapshot.you);
                self.game = Some(AnyGameSnapshot::QiGui523(snapshot));
                self.lobby = None;
                self.last_rejection = None;
            }
            ServerEvent::GameSnapshot(AnyGameSnapshot::TexasHoldem(snapshot)) => {
                self.host_port = Some(snapshot.host_port);
                if self.active_match_id != Some(snapshot.match_id) {
                    self.pending_texas_holdem_events.clear();
                }
                if let TexasHoldemPhaseView::HandComplete {
                    tournament_complete: true,
                    reference_changes,
                    ..
                } = &snapshot.phase
                {
                    self.last_finished_match = Some((snapshot.match_id, reference_changes.clone()));
                }
                self.active_match_id = Some(snapshot.match_id);
                self.you = Some(snapshot.you);
                self.game = Some(AnyGameSnapshot::TexasHoldem(snapshot));
                self.lobby = None;
                self.last_rejection = None;
            }
            ServerEvent::GameSnapshot(AnyGameSnapshot::Shengji(snapshot)) => {
                self.host_port = Some(snapshot.host_port);
                if self.active_match_id != Some(snapshot.match_id) {
                    self.pending_shengji_events.clear();
                    self.shengji_collected_score_cards.clear();
                    self.shengji_pending_trick_score_cards.clear();
                    self.shengji_finished_event_pending_snapshot = false;
                    self.last_shengji_score_capture = None;
                    self.shengji_score_capture_serial = 0;
                } else {
                    if self
                        .shengji_game()
                        .is_some_and(|previous| previous.hand_number != snapshot.hand_number)
                    {
                        self.shengji_collected_score_cards.clear();
                        self.shengji_pending_trick_score_cards.clear();
                        self.shengji_finished_event_pending_snapshot = false;
                        self.last_shengji_score_capture = None;
                        self.shengji_score_capture_serial = 0;
                    }
                    let captured = if self.shengji_finished_event_pending_snapshot {
                        self.shengji_finished_event_pending_snapshot = false;
                        Vec::new()
                    } else {
                        self.shengji_game().map_or_else(Vec::new, |previous| {
                            completed_shengji_trick_score_cards(previous, &snapshot)
                        })
                    };
                    for card in captured {
                        if !self.shengji_collected_score_cards.contains(&card) {
                            self.shengji_collected_score_cards.push(card);
                        }
                    }
                }
                if let ShengjiPhaseView::Finished { result, .. } = &snapshot.phase {
                    self.last_finished_match =
                        Some((result.settlement_id, result.reference_changes.clone()));
                }
                self.active_match_id = Some(snapshot.match_id);
                self.you = Some(snapshot.you);
                self.rules = Some(GameRules::Shengji(snapshot.rules));
                self.game = Some(AnyGameSnapshot::Shengji(snapshot));
                self.lobby = None;
                self.last_rejection = None;
            }
            ServerEvent::GameSnapshot(AnyGameSnapshot::Uno(snapshot)) => {
                self.host_port = Some(snapshot.host_port);
                if self.active_match_id != Some(snapshot.match_id) {
                    self.pending_uno_events.clear();
                }
                if let UnoPhaseView::Finished {
                    reference_changes, ..
                } = &snapshot.phase
                {
                    self.last_finished_match = Some((snapshot.match_id, reference_changes.clone()));
                }
                self.active_match_id = Some(snapshot.match_id);
                self.you = Some(snapshot.you);
                self.rules = Some(GameRules::Uno(snapshot.rules));
                self.game = Some(AnyGameSnapshot::Uno(snapshot));
                self.lobby = None;
                self.last_rejection = None;
            }
            ServerEvent::GameEvent(GameEvent::QiGui523(QiGui523Event::PlayEffect {
                player,
                play,
            })) => {
                self.play_effect_serial = self.play_effect_serial.saturating_add(1);
                self.last_play_effect = Some((player, play));
            }
            ServerEvent::GameEvent(GameEvent::TexasHoldem(event)) => {
                self.pending_texas_holdem_events.push_back(event);
            }
            ServerEvent::GameEvent(GameEvent::Shengji(event)) => {
                match &event {
                    ShengjiEvent::CardsPlayed { play, .. } => {
                        for card in play
                            .play
                            .cards
                            .iter()
                            .copied()
                            .filter(|card| card.points() > 0)
                        {
                            if !self.shengji_pending_trick_score_cards.contains(&card) {
                                self.shengji_pending_trick_score_cards.push(card);
                            }
                        }
                    }
                    ShengjiEvent::TrickFinished {
                        winner,
                        collecting_score,
                        ..
                    } => {
                        let collecting_side_won = self
                            .shengji_game()
                            .and_then(|game| game.dealer)
                            .is_some_and(|dealer| winner.0 % 2 != dealer.0 % 2);
                        if collecting_side_won {
                            let cards = self
                                .shengji_pending_trick_score_cards
                                .drain(..)
                                .collect::<Vec<_>>();
                            if !cards.is_empty() {
                                let score_before =
                                    self.shengji_game().map_or(0, |game| game.collecting_score);
                                self.shengji_score_capture_serial =
                                    self.shengji_score_capture_serial.saturating_add(1);
                                self.last_shengji_score_capture = Some(ShengjiScoreCaptureEffect {
                                    cards: cards.clone(),
                                    score_before,
                                    score_after: *collecting_score,
                                });
                            }
                            for card in cards {
                                if !self.shengji_collected_score_cards.contains(&card) {
                                    self.shengji_collected_score_cards.push(card);
                                }
                            }
                        } else {
                            self.shengji_pending_trick_score_cards.clear();
                        }
                        self.shengji_finished_event_pending_snapshot = true;
                    }
                    ShengjiEvent::RedealRequired => {
                        self.shengji_pending_trick_score_cards.clear();
                        self.shengji_finished_event_pending_snapshot = false;
                    }
                    _ => {}
                }
                self.pending_shengji_events.push_back(event);
            }
            ServerEvent::GameEvent(GameEvent::Uno(event)) => {
                if let Some(notice) = uno_event_notice(self.uno_game(), &event) {
                    self.notice_serial = self.notice_serial.saturating_add(1);
                    self.last_notice = Some(notice);
                }
                self.pending_uno_events.push_back(event);
            }
            ServerEvent::PlayerInteraction(interaction) => {
                self.pending_player_interactions.push_back(interaction);
            }
            ServerEvent::ChatMessage(message) => {
                self.pending_chat_messages.push_back(message);
            }
            ServerEvent::PlayerLeft { name } => {
                self.notice_serial = self.notice_serial.saturating_add(1);
                self.last_notice = Some(format!("{name}退出了游戏"));
            }
            ServerEvent::LeftRoom => {
                self.left_room = true;
            }
            ServerEvent::RoomClosed => {
                self.room_closed = true;
            }
            ServerEvent::Rejected { reason } => {
                self.rejection_serial = self.rejection_serial.saturating_add(1);
                self.last_rejection = Some(reason);
            }
            ServerEvent::Heartbeat => {}
        }
        true
    }

    pub fn room_id(&self) -> RoomId {
        self.room_id
    }

    pub fn host_port(&self) -> Option<u16> {
        self.host_port
    }

    pub fn latest_revision(&self) -> Revision {
        self.latest_revision
    }

    pub fn you(&self) -> Option<PlayerId> {
        self.you
    }

    pub fn lobby(&self) -> Option<&LobbySnapshot> {
        self.lobby.as_ref()
    }

    pub fn game_snapshot(&self) -> Option<&AnyGameSnapshot> {
        self.game.as_ref()
    }

    pub fn qigui523_game(&self) -> Option<&GameSnapshot> {
        self.game.as_ref().and_then(AnyGameSnapshot::qigui523)
    }

    pub fn texas_holdem_game(&self) -> Option<&TexasHoldemSnapshot> {
        self.game.as_ref().and_then(AnyGameSnapshot::texas_holdem)
    }

    pub fn shengji_game(&self) -> Option<&ShengjiSnapshot> {
        self.game.as_ref().and_then(AnyGameSnapshot::shengji)
    }

    pub fn uno_game(&self) -> Option<&UnoSnapshot> {
        self.game.as_ref().and_then(AnyGameSnapshot::uno)
    }

    pub fn game_rules(&self) -> Option<&GameRules> {
        self.rules.as_ref()
    }

    pub fn qigui523_rules(&self) -> Option<&RuleSet> {
        self.rules.as_ref().and_then(GameRules::qigui523)
    }

    pub fn texas_holdem_rules(&self) -> Option<&TexasHoldemRuleSet> {
        self.rules.as_ref().and_then(GameRules::texas_holdem)
    }

    pub fn shengji_rules(&self) -> Option<&ShengjiRuleSet> {
        self.rules.as_ref().and_then(GameRules::shengji)
    }

    pub fn uno_rules(&self) -> Option<&UnoRuleSet> {
        self.rules.as_ref().and_then(GameRules::uno)
    }

    pub fn avatars(&self) -> &HashMap<AvatarId, Vec<u8>> {
        &self.avatars
    }

    /// 当前这一局中，各玩家已经收入的全部 5、10、K。
    ///
    /// 这份历史完全由客户端从公开的每轮出牌快照推导，不进入网络协议。
    pub fn captured_score_cards(&self, player: PlayerId) -> &[Card] {
        self.captured_score_cards
            .get(&player)
            .map(Vec::as_slice)
            .unwrap_or_default()
    }

    /// 当前双升牌局中闲家已经收走的公开 5、10、K 实体牌。
    pub fn shengji_collected_score_cards(&self) -> &[leocard_shengji::Card] {
        &self.shengji_collected_score_cards
    }

    pub fn last_shengji_score_capture(&self) -> Option<&ShengjiScoreCaptureEffect> {
        self.last_shengji_score_capture.as_ref()
    }

    pub fn shengji_score_capture_serial(&self) -> u64 {
        self.shengji_score_capture_serial
    }

    pub fn room_closed(&self) -> bool {
        self.room_closed
    }

    pub fn left_room(&self) -> bool {
        self.left_room
    }

    pub fn last_notice(&self) -> Option<&str> {
        self.last_notice.as_deref()
    }

    pub fn notice_serial(&self) -> u64 {
        self.notice_serial
    }

    pub fn last_play_effect(&self) -> Option<&(PlayerId, PublicPlay)> {
        self.last_play_effect.as_ref()
    }

    pub fn play_effect_serial(&self) -> u64 {
        self.play_effect_serial
    }

    pub fn last_score_capture(&self) -> Option<&ScoreCaptureEffect> {
        self.last_score_capture.as_ref()
    }

    pub fn score_capture_serial(&self) -> u64 {
        self.score_capture_serial
    }

    pub fn take_player_interactions(&mut self) -> Vec<PlayerInteraction> {
        self.pending_player_interactions.drain(..).collect()
    }

    pub fn take_chat_messages(&mut self) -> Vec<ChatMessage> {
        self.pending_chat_messages.drain(..).collect()
    }

    pub fn take_texas_holdem_events(&mut self) -> Vec<TexasHoldemEvent> {
        self.pending_texas_holdem_events.drain(..).collect()
    }

    pub fn take_shengji_events(&mut self) -> Vec<ShengjiEvent> {
        self.pending_shengji_events.drain(..).collect()
    }

    pub fn take_uno_events(&mut self) -> Vec<UnoEvent> {
        self.pending_uno_events.drain(..).collect()
    }

    pub fn last_finished_match(&self) -> Option<(MatchId, &[PlayerReferenceChange])> {
        self.last_finished_match
            .as_ref()
            .map(|(match_id, changes)| (*match_id, changes.as_slice()))
    }

    pub fn last_rejection(&self) -> Option<&RejectReason> {
        self.last_rejection.as_ref()
    }

    /// 每收到一次拒绝消息便递增，即使相邻两次拒绝的内容完全相同。
    pub fn rejection_serial(&self) -> u64 {
        self.rejection_serial
    }

    fn observe_score_cards(&mut self, snapshot: &GameSnapshot) {
        if let GamePhaseView::Finished {
            finisher,
            ref remaining_hands,
            ..
        } = snapshot.phase
        {
            let is_new_finish = !self.qigui523_game().is_some_and(|game| {
                game.match_id == snapshot.match_id
                    && matches!(game.phase, GamePhaseView::Finished { .. })
            });
            if is_new_finish {
                let mut already_captured = self
                    .captured_score_cards
                    .values()
                    .flatten()
                    .copied()
                    .collect::<std::collections::HashSet<_>>();
                let mut visible_cards = self
                    .observed_trick
                    .as_ref()
                    .into_iter()
                    .flat_map(|trick| trick.records.iter())
                    .flat_map(score_cards_in_record)
                    .filter(|card| already_captured.insert(*card))
                    .collect::<Vec<_>>();
                if let Some((player, play)) = &self.last_play_effect
                    && *player == finisher
                {
                    for card in play.cards.iter().copied().filter(|card| card.score() > 0) {
                        if already_captured.insert(card) {
                            visible_cards.push(card);
                        }
                    }
                }
                let mut source_players = vec![None; visible_cards.len()];
                for hand in remaining_hands {
                    for card in hand.cards.iter().copied().filter(|card| card.score() > 0) {
                        if !already_captured.insert(card) {
                            continue;
                        }
                        visible_cards.push(card);
                        source_players.push(Some(hand.player));
                    }
                }
                self.record_score_capture(
                    snapshot,
                    finisher,
                    visible_cards.clone(),
                    source_players,
                );
                self.captured_score_cards
                    .entry(finisher)
                    .or_default()
                    .extend(visible_cards);
            }
            self.observed_trick = None;
            return;
        }

        if let Some(previous) = self.observed_trick.take() {
            let is_same_trick = snapshot.trick.as_ref().is_some_and(|current| {
                current.leader == previous.leader && current.records.starts_with(&previous.records)
            });
            if !is_same_trick && let Some(winner) = previous.winning_player {
                let captured = previous
                    .records
                    .iter()
                    .flat_map(score_cards_in_record)
                    .collect::<Vec<_>>();
                self.record_score_capture(
                    snapshot,
                    winner,
                    captured.clone(),
                    vec![None; captured.len()],
                );
                self.captured_score_cards
                    .entry(winner)
                    .or_default()
                    .extend(captured);
            }
        }
        self.observed_trick.clone_from(&snapshot.trick);
    }

    fn record_score_capture(
        &mut self,
        snapshot: &GameSnapshot,
        player: PlayerId,
        cards: Vec<Card>,
        source_players: Vec<Option<PlayerId>>,
    ) {
        if cards.is_empty() {
            return;
        }
        let captured_points = cards
            .iter()
            .copied()
            .map(Card::score)
            .map(u32::from)
            .sum::<u32>();
        let score_after = snapshot
            .players
            .iter()
            .find(|state| state.id == player)
            .map_or(captured_points, |state| state.score);
        let score_before = self
            .qigui523_game()
            .and_then(|game| game.players.iter().find(|state| state.id == player))
            .map_or_else(
                || score_after.saturating_sub(captured_points),
                |state| state.score,
            );
        self.score_capture_serial = self.score_capture_serial.saturating_add(1);
        self.last_score_capture = Some(ScoreCaptureEffect {
            player,
            cards,
            source_players,
            score_before,
            score_after,
        });
    }
}

fn uno_event_notice(snapshot: Option<&UnoSnapshot>, event: &UnoEvent) -> Option<String> {
    let player_name = |player: leocard_protocol::PlayerId| {
        snapshot
            .and_then(|snapshot| snapshot.players.iter().find(|item| item.id == player))
            .map(|player| player.name.clone())
            .unwrap_or_else(|| format!("玩家 {}", player.0 + 1))
    };
    match event {
        UnoEvent::ChallengeResolved {
            challenger,
            offender,
            result,
            penalized,
            count,
        } => Some(match result {
            leocard_uno::ChallengeResult::Successful => format!(
                "{} 质疑成功，{} 摸 {count} 张",
                player_name(*challenger),
                player_name(*offender)
            ),
            leocard_uno::ChallengeResult::Failed => format!(
                "{} 质疑失败，{} 摸 {count} 张",
                player_name(*challenger),
                player_name(*penalized)
            ),
        }),
        UnoEvent::UnoCalled { player } => Some(format!("{}：UNO!", player_name(*player))),
        UnoEvent::UnoReported { reporter, target } => Some(format!(
            "{} 检举了 {}，罚摸 2 张",
            player_name(*reporter),
            player_name(*target)
        )),
        UnoEvent::SkipResolved { .. } => None,
        UnoEvent::ColorChosen { .. }
        | UnoEvent::CardPlayed { .. }
        | UnoEvent::CardsDrawn { .. }
        | UnoEvent::GameFinished { .. } => None,
    }
}

fn score_cards_in_record(record: &PublicPlayRecord) -> Vec<Card> {
    match record {
        PublicPlayRecord::Played { play, .. } => play
            .cards
            .iter()
            .copied()
            .filter(|card| card.score() > 0)
            .collect(),
        PublicPlayRecord::Passed { .. } => Vec::new(),
    }
}

fn completed_shengji_trick_score_cards(
    previous: &ShengjiSnapshot,
    next: &ShengjiSnapshot,
) -> Vec<leocard_shengji::Card> {
    let Some(trick) = previous.trick.as_ref() else {
        return Vec::new();
    };
    let same_trick_continues = next.trick.as_ref().is_some_and(|current| {
        current.leader == trick.leader
            && current.plays.len() >= trick.plays.len()
            && current.plays[..trick.plays.len()] == trick.plays
    });
    let just_finished = matches!(
        next.phase,
        leocard_protocol::ShengjiPhaseView::Finished { .. }
    ) && !matches!(
        previous.phase,
        leocard_protocol::ShengjiPhaseView::Finished { .. }
    );
    if same_trick_continues && !just_finished {
        return Vec::new();
    }
    let Some(dealer) = previous.dealer.or(next.dealer) else {
        return Vec::new();
    };
    if trick.winning_player.0 % 2 == dealer.0 % 2 {
        return Vec::new();
    }
    trick
        .plays
        .iter()
        .flat_map(|played| played.play.cards.iter().copied())
        .filter(|card| card.points() > 0)
        .collect()
}

/// 仅供单元测试使用的本地演示适配器；正式 Bevy 客户端使用 [`TcpGameClient`]。
pub struct InMemoryClient {
    connection: ConnectionId,
    model: ClientModel,
    host: HostSession,
    remote_next_request: [u64; 3],
}

impl InMemoryClient {
    /// 创建三人演示房间；另外两个玩家只用于让房间能够开局。
    pub fn demo(local_name: &str) -> Result<Self, HostError> {
        let room_id = RoomId(523);
        let rules = RuleSet {
            player_count: 6,
            ..RuleSet::default()
        };
        let mut client = Self {
            connection: ConnectionId(1),
            model: ClientModel::new(room_id),
            host: HostSession::qigui523(room_id, 52300, rules, build_deck(rules.deck_count))?,
            remote_next_request: [2, 4, 4],
        };

        client.send(test_join_command(local_name, ReconnectToken(1)));
        client.send(ClientCommand::SelectSeat { seat: SeatId(0) });
        for (connection, name) in [(ConnectionId(2), "Demo 2"), (ConnectionId(3), "Demo 3")] {
            let join = ClientMessage::new(
                room_id,
                RequestId(1),
                test_join_command(name, ReconnectToken(connection.0)),
            );
            let deliveries = client.host.handle(connection, join);
            client.deliver(deliveries);
            let select_seat = ClientMessage::new(
                room_id,
                RequestId(2),
                ClientCommand::SelectSeat {
                    seat: SeatId(connection.0 as u8 - 1),
                },
            );
            let deliveries = client.host.handle(connection, select_seat);
            client.deliver(deliveries);
            let ready = ClientMessage::new(
                room_id,
                RequestId(3),
                ClientCommand::SetReady { ready: true },
            );
            let deliveries = client.host.handle(connection, ready);
            client.deliver(deliveries);
        }
        Ok(client)
    }

    pub fn model(&self) -> &ClientModel {
        &self.model
    }

    pub fn send(&mut self, command: ClientCommand) {
        let message = self.model.command(command);
        let deliveries = self.host.handle(self.connection, message);
        self.deliver(deliveries);
        self.advance_demo_players();
    }

    fn deliver(&mut self, deliveries: Vec<leocard_host::Delivery>) {
        for delivery in deliveries {
            if delivery.recipient == self.connection {
                self.model.apply(delivery.message);
            }
        }
    }

    /// 演示玩家永远过牌，使本地玩家可以持续验证多轮 UI；所有操作仍走正式协议。
    fn advance_demo_players(&mut self) {
        for _ in 0..16 {
            let Some(current) = self
                .model
                .qigui523_game()
                .and_then(|game| game.trick.as_ref())
                .map(|trick| trick.current_player)
            else {
                return;
            };
            if current == PlayerId(0) {
                return;
            }

            let index = usize::from(current.0);
            let request_id = RequestId(self.remote_next_request[index]);
            self.remote_next_request[index] += 1;
            let message = ClientMessage::new(
                self.model.room_id(),
                request_id,
                ClientCommand::Game(GameCommand::QiGui523(QiGui523Command::Pass)),
            );
            let before = self.host.revision();
            let deliveries = self
                .host
                .handle(ConnectionId(u64::from(current.0) + 1), message);
            self.deliver(deliveries);
            if self.host.revision() == before {
                return;
            }
        }
    }
}

fn test_join_command(name: &str, token: ReconnectToken) -> ClientCommand {
    let mut secret = [0; 32];
    secret[..8].copy_from_slice(&token.0.to_be_bytes());
    secret[8] = 1;
    PlayerIdentity::from_secret_bytes(secret).join_command(
        NETWORK_ROOM_ID,
        name,
        token,
        0,
        0,
        PlayerGameProfiles::default(),
    )
}

pub fn selected_cards_in_hand(selected: &[Card], snapshot: &GameSnapshot) -> Vec<Card> {
    selected
        .iter()
        .copied()
        .filter(|card| snapshot.your_hand.contains(card))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use leocard_protocol::{ChatContent, GameKind, GameViolation, PlayerInteractionKind};
    use std::net::TcpListener as StdTcpListener;
    use std::time::{Duration, Instant};

    #[test]
    fn join_address_accepts_dns_names_and_ip_literals() {
        assert_eq!(
            normalize_server_address("frp-off.com:52436").unwrap(),
            "frp-off.com:52436"
        );
        assert_eq!(
            normalize_server_address(" 192.168.1.8:52300 ").unwrap(),
            "192.168.1.8:52300"
        );
        assert_eq!(
            normalize_server_address("[2001:db8::1]:52300").unwrap(),
            "[2001:db8::1]:52300"
        );
    }

    #[test]
    fn join_address_rejects_missing_or_invalid_ports() {
        for address in [
            "frp-off.com",
            "frp-off.com:0",
            "frp-off.com:70000",
            ":52300",
        ] {
            assert!(normalize_server_address(address).is_err(), "{address}");
        }
    }

    fn score_history_snapshot(trick: Option<TrickView>, phase: GamePhaseView) -> GameSnapshot {
        let match_id = match &phase {
            GamePhaseView::Finished { match_id, .. } => *match_id,
            GamePhaseView::Playing => MatchId([0; 16]),
        };
        GameSnapshot {
            match_id,
            host_port: 52300,
            you: PlayerId(0),
            host: PlayerId(0),
            players: Vec::new(),
            your_hand: Vec::new(),
            draw_pile_len: 0,
            starting_card: leocard_protocol::StartingCardView {
                player: PlayerId(0),
                card: Card::suited(
                    0,
                    leocard_qigui523::Suit::Diamond,
                    leocard_qigui523::Rank::Four,
                ),
            },
            trick,
            turn_timer: None,
            phase,
        }
    }

    #[test]
    fn client_records_all_score_cards_when_a_trick_changes() {
        let five = Card::suited(
            0,
            leocard_qigui523::Suit::Diamond,
            leocard_qigui523::Rank::Five,
        );
        let ten = Card::suited(0, leocard_qigui523::Suit::Club, leocard_qigui523::Rank::Ten);
        let mut model = ClientModel::new(RoomId(7));
        let completed = TrickView {
            leader: PlayerId(0),
            current_player: PlayerId(0),
            winning_player: Some(PlayerId(1)),
            winning_play: None,
            records: vec![
                PublicPlayRecord::Played {
                    player: PlayerId(0),
                    play: leocard_protocol::PublicPlay {
                        kind: leocard_qigui523::PlayKind::Single,
                        cards: vec![five],
                    },
                },
                PublicPlayRecord::Played {
                    player: PlayerId(1),
                    play: leocard_protocol::PublicPlay {
                        kind: leocard_qigui523::PlayKind::Single,
                        cards: vec![ten],
                    },
                },
            ],
            table_points: 15,
        };
        model.observe_score_cards(&score_history_snapshot(
            Some(completed),
            GamePhaseView::Playing,
        ));
        model.observe_score_cards(&score_history_snapshot(
            Some(TrickView {
                leader: PlayerId(1),
                current_player: PlayerId(1),
                winning_player: None,
                winning_play: None,
                records: Vec::new(),
                table_points: 0,
            }),
            GamePhaseView::Playing,
        ));

        assert_eq!(model.captured_score_cards(PlayerId(1)), &[five, ten]);
        assert!(model.captured_score_cards(PlayerId(0)).is_empty());
        assert_eq!(model.score_capture_serial(), 1);
        assert_eq!(
            model.last_score_capture(),
            Some(&ScoreCaptureEffect {
                player: PlayerId(1),
                cards: vec![five, ten],
                source_players: vec![None, None],
                score_before: 0,
                score_after: 15,
            })
        );
    }

    #[test]
    fn client_assigns_revealed_hand_score_cards_to_the_finisher_once() {
        let five = Card::suited(
            0,
            leocard_qigui523::Suit::Diamond,
            leocard_qigui523::Rank::Five,
        );
        let ten = Card::suited(0, leocard_qigui523::Suit::Club, leocard_qigui523::Rank::Ten);
        let king = Card::suited(
            0,
            leocard_qigui523::Suit::Heart,
            leocard_qigui523::Rank::King,
        );
        let mut model = ClientModel::new(RoomId(7));
        model.captured_score_cards.insert(PlayerId(0), vec![five]);
        let finished = score_history_snapshot(
            None,
            GamePhaseView::Finished {
                match_id: leocard_protocol::MatchId([1; 16]),
                finisher: PlayerId(1),
                scores: Vec::new(),
                remaining_hands: vec![leocard_protocol::RevealedHand {
                    player: PlayerId(0),
                    cards: vec![ten, king],
                }],
                reference_changes: Vec::new(),
                captured_hand_points: 20,
            },
        );

        model.observe_score_cards(&finished);
        model.observe_score_cards(&finished);

        assert_eq!(model.captured_score_cards(PlayerId(0)), &[five]);
        assert_eq!(model.captured_score_cards(PlayerId(1)), &[ten, king]);
        assert_eq!(
            model
                .last_score_capture()
                .map(|effect| effect.source_players.as_slice()),
            Some(&[Some(PlayerId(0)), Some(PlayerId(0))][..])
        );
    }

    #[test]
    fn client_clears_score_card_history_when_a_rematch_starts() {
        let mut model = ClientModel::new(RoomId(7));
        let five = Card::suited(
            0,
            leocard_qigui523::Suit::Diamond,
            leocard_qigui523::Rank::Five,
        );
        model.captured_score_cards.insert(PlayerId(0), vec![five]);
        model.active_match_id = Some(MatchId([2; 16]));
        let mut previous = score_history_snapshot(None, GamePhaseView::Playing);
        previous.match_id = MatchId([2; 16]);
        model.game = Some(AnyGameSnapshot::QiGui523(previous));
        let mut next = score_history_snapshot(None, GamePhaseView::Playing);
        next.match_id = MatchId([3; 16]);

        assert!(model.apply(ServerMessage {
            protocol_version: PROTOCOL_VERSION,
            room_id: RoomId(7),
            revision: Revision(1),
            in_reply_to: None,
            event: ServerEvent::GameSnapshot(AnyGameSnapshot::QiGui523(next)),
        }));

        assert!(model.captured_score_cards(PlayerId(0)).is_empty());
        assert!(model.observed_trick.is_none());
        assert_eq!(model.active_match_id, Some(MatchId([3; 16])));
    }

    fn shengji_score_snapshot(
        trick: Option<leocard_protocol::ShengjiTrickView>,
    ) -> ShengjiSnapshot {
        ShengjiSnapshot {
            match_id: MatchId([6; 16]),
            hand_number: 1,
            host_port: 52300,
            you: PlayerId(0),
            host: PlayerId(0),
            rules: ShengjiRuleSet::default(),
            players: (0..4)
                .map(|id| leocard_protocol::ShengjiPlayerState {
                    id: PlayerId(id),
                    profile_id: ProfileId([id; 32]),
                    name: format!("玩家{id}"),
                    avatar: None,
                    seat: SeatId(id),
                    hand_len: 24,
                    ready: false,
                    connected: true,
                    auto_play: false,
                    reference_points: 0,
                    completed_games: 0,
                    game_profiles: PlayerGameProfiles::default(),
                })
                .collect(),
            your_hand: Vec::new(),
            your_exposed_cards: Vec::new(),
            levels: [leocard_shengji::Rank::Ten; 2],
            bidding_level: leocard_shengji::Rank::Ten,
            dealer: Some(PlayerId(0)),
            trump: Some(
                leocard_shengji::Trump::new(
                    leocard_shengji::Rank::Ten,
                    Some(leocard_shengji::Suit::Heart),
                )
                .unwrap(),
            ),
            declaration: None,
            current_player: Some(PlayerId(0)),
            trick,
            throw_failure: None,
            collecting_score: 15,
            buried_count: 8,
            your_buried: Vec::new(),
            phase: leocard_protocol::ShengjiPhaseView::Playing,
        }
    }

    #[test]
    fn client_keeps_the_collecting_sides_public_shengji_score_cards() {
        let five = leocard_shengji::Card::suited(
            0,
            leocard_shengji::Suit::Club,
            leocard_shengji::Rank::Five,
        );
        let ten = leocard_shengji::Card::suited(
            0,
            leocard_shengji::Suit::Spade,
            leocard_shengji::Rank::Ten,
        );
        let previous = shengji_score_snapshot(Some(leocard_protocol::ShengjiTrickView {
            leader: PlayerId(0),
            current_player: PlayerId(0),
            winning_player: PlayerId(1),
            plays: vec![leocard_protocol::ShengjiPublicPlay {
                player: PlayerId(1),
                play: leocard_shengji::ClassifiedPlay {
                    cards: vec![five, ten],
                    category: leocard_shengji::Category::Suit(leocard_shengji::Suit::Club),
                    components: Vec::new(),
                },
                throw_penalty: 0,
            }],
            table_points: 15,
        }));
        let next = shengji_score_snapshot(None);
        let mut model = ClientModel::new(RoomId(7));

        assert!(model.apply(ServerMessage {
            protocol_version: PROTOCOL_VERSION,
            room_id: RoomId(7),
            revision: Revision(1),
            in_reply_to: None,
            event: ServerEvent::GameSnapshot(AnyGameSnapshot::Shengji(previous)),
        }));
        assert!(model.apply(ServerMessage {
            protocol_version: PROTOCOL_VERSION,
            room_id: RoomId(7),
            revision: Revision(2),
            in_reply_to: None,
            event: ServerEvent::GameSnapshot(AnyGameSnapshot::Shengji(next)),
        }));

        assert_eq!(model.shengji_collected_score_cards(), &[five, ten]);
    }

    #[test]
    fn shengji_trick_events_keep_the_fourth_players_score_card() {
        let five = leocard_shengji::Card::suited(
            0,
            leocard_shengji::Suit::Club,
            leocard_shengji::Rank::Five,
        );
        let ten = leocard_shengji::Card::suited(
            0,
            leocard_shengji::Suit::Club,
            leocard_shengji::Rank::Ten,
        );
        let mut model = ClientModel::new(RoomId(7));
        assert!(model.apply(ServerMessage {
            protocol_version: PROTOCOL_VERSION,
            room_id: RoomId(7),
            revision: Revision(1),
            in_reply_to: None,
            event: ServerEvent::GameSnapshot(AnyGameSnapshot::Shengji(shengji_score_snapshot(
                None,
            ))),
        }));
        for (revision, player, card) in [(2, 0, five), (3, 3, ten)] {
            assert!(model.apply(ServerMessage {
                protocol_version: PROTOCOL_VERSION,
                room_id: RoomId(7),
                revision: Revision(revision),
                in_reply_to: None,
                event: ServerEvent::GameEvent(GameEvent::Shengji(ShengjiEvent::CardsPlayed {
                    play: leocard_protocol::ShengjiPublicPlay {
                        player: PlayerId(player),
                        play: leocard_shengji::ClassifiedPlay {
                            cards: vec![card],
                            category: leocard_shengji::Category::Suit(leocard_shengji::Suit::Club,),
                            components: Vec::new(),
                        },
                        throw_penalty: 0,
                    },
                    is_lead: player == 0,
                })),
            }));
        }
        assert!(model.apply(ServerMessage {
            protocol_version: PROTOCOL_VERSION,
            room_id: RoomId(7),
            revision: Revision(4),
            in_reply_to: None,
            event: ServerEvent::GameEvent(GameEvent::Shengji(ShengjiEvent::TrickFinished {
                winner: PlayerId(3),
                points: 15,
                collecting_score: 15,
            })),
        }));

        assert_eq!(model.shengji_collected_score_cards(), &[five, ten]);
    }

    #[test]
    fn shengji_finish_event_prevents_an_incomplete_snapshot_from_awarding_the_wrong_team() {
        let five = leocard_shengji::Card::suited(
            0,
            leocard_shengji::Suit::Club,
            leocard_shengji::Rank::Five,
        );
        let play = leocard_protocol::ShengjiPublicPlay {
            player: PlayerId(1),
            play: leocard_shengji::ClassifiedPlay {
                cards: vec![five],
                category: leocard_shengji::Category::Suit(leocard_shengji::Suit::Club),
                components: Vec::new(),
            },
            throw_penalty: 0,
        };
        let mut model = ClientModel::new(RoomId(7));
        for (revision, event) in [
            (
                1,
                ServerEvent::GameSnapshot(AnyGameSnapshot::Shengji(shengji_score_snapshot(None))),
            ),
            (
                2,
                ServerEvent::GameEvent(GameEvent::Shengji(ShengjiEvent::CardsPlayed {
                    play: play.clone(),
                    is_lead: true,
                })),
            ),
            (
                3,
                ServerEvent::GameSnapshot(AnyGameSnapshot::Shengji(shengji_score_snapshot(Some(
                    leocard_protocol::ShengjiTrickView {
                        leader: PlayerId(1),
                        current_player: PlayerId(0),
                        winning_player: PlayerId(1),
                        plays: vec![play],
                        table_points: 5,
                    },
                )))),
            ),
            (
                4,
                ServerEvent::GameEvent(GameEvent::Shengji(ShengjiEvent::TrickFinished {
                    winner: PlayerId(0),
                    points: 5,
                    collecting_score: 0,
                })),
            ),
            (
                5,
                ServerEvent::GameSnapshot(AnyGameSnapshot::Shengji(shengji_score_snapshot(None))),
            ),
        ] {
            assert!(model.apply(ServerMessage {
                protocol_version: PROTOCOL_VERSION,
                room_id: RoomId(7),
                revision: Revision(revision),
                in_reply_to: None,
                event,
            }));
        }

        assert!(model.shengji_collected_score_cards().is_empty());
    }

    #[test]
    fn finished_match_receipt_survives_a_following_lobby_snapshot() {
        let mut model = ClientModel::new(RoomId(7));
        let match_id = MatchId([9; 16]);
        let profile_id = ProfileId([4; 32]);
        let change = PlayerReferenceChange {
            player: PlayerId(0),
            profile_id,
            delta: 2,
        };
        assert!(model.apply(ServerMessage {
            protocol_version: PROTOCOL_VERSION,
            room_id: RoomId(7),
            revision: Revision(1),
            in_reply_to: None,
            event: ServerEvent::GameSnapshot(AnyGameSnapshot::QiGui523(score_history_snapshot(
                None,
                GamePhaseView::Finished {
                    match_id,
                    finisher: PlayerId(0),
                    scores: Vec::new(),
                    remaining_hands: Vec::new(),
                    reference_changes: vec![change],
                    captured_hand_points: 0,
                },
            ))),
        }));
        assert!(model.apply(ServerMessage {
            protocol_version: PROTOCOL_VERSION,
            room_id: RoomId(7),
            revision: Revision(2),
            in_reply_to: None,
            event: ServerEvent::LobbySnapshot(LobbySnapshot {
                game: GameKind::QiGui523,
                rules: GameRules::QiGui523(RuleSet::default()),
                host_port: 52300,
                host: None,
                players: Vec::new(),
            }),
        }));

        assert_eq!(model.last_finished_match(), Some((match_id, &[change][..])));
    }

    #[test]
    fn demo_client_reaches_lobby_and_starts_only_through_protocol_commands() {
        let mut client = InMemoryClient::demo("Local").unwrap();
        assert_eq!(client.model().lobby().unwrap().players.len(), 3);

        client.send(ClientCommand::SetReady { ready: true });
        client.send(ClientCommand::StartGame);

        let game = client.model().qigui523_game().unwrap();
        assert_eq!(game.you, PlayerId(0));
        assert_eq!(game.your_hand.len(), 5);
        assert_eq!(game.players.len(), 3);
        assert!(game.players.iter().all(|player| player.hand_len == 5));

        let card = game.your_hand[0];
        client.send(ClientCommand::Game(GameCommand::QiGui523(
            QiGui523Command::PlayCards { cards: vec![card] },
        )));
        let after_round = client.model().qigui523_game().unwrap();
        assert_eq!(
            after_round.trick.as_ref().unwrap().current_player,
            PlayerId(0)
        );
        assert_eq!(
            after_round.your_hand.len(),
            if cfg!(feature = "developer") { 4 } else { 5 }
        );
    }

    #[test]
    fn client_ignores_old_or_foreign_snapshots() {
        let mut client = InMemoryClient::demo("Local").unwrap();
        let before = client.model().latest_revision();
        let mut foreign = ServerMessage {
            protocol_version: PROTOCOL_VERSION,
            room_id: RoomId(999),
            revision: Revision(before.0 + 100),
            in_reply_to: None,
            event: ServerEvent::Joined { you: PlayerId(7) },
        };
        assert!(!client.model.apply(foreign.clone()));
        foreign.room_id = client.model.room_id();
        foreign.revision = Revision(before.0.saturating_sub(1));
        assert!(!client.model.apply(foreign));
        assert_eq!(client.model().you(), Some(PlayerId(0)));
    }

    #[test]
    fn avatar_payload_is_cached_independently_from_snapshots() {
        let mut model = ClientModel::new(RoomId(7));
        let png = vec![1, 2, 3, 4];
        assert!(model.apply(ServerMessage {
            protocol_version: PROTOCOL_VERSION,
            room_id: RoomId(7),
            revision: Revision(1),
            in_reply_to: None,
            event: ServerEvent::AvatarData {
                id: AvatarId(9),
                png: png.clone(),
            },
        }));
        assert_eq!(model.avatars().get(&AvatarId(9)), Some(&png));
        assert!(model.lobby().is_none());
        assert!(model.qigui523_game().is_none());
    }

    #[test]
    fn repeated_identical_rejections_each_advance_the_local_serial() {
        let mut model = ClientModel::new(RoomId(7));
        let rejection = ServerMessage {
            protocol_version: PROTOCOL_VERSION,
            room_id: RoomId(7),
            revision: Revision(1),
            in_reply_to: None,
            event: ServerEvent::Rejected {
                reason: RejectReason::GameViolation(GameViolation::QiGui523(
                    leocard_protocol::RuleViolation::InvalidPattern,
                )),
            },
        };

        assert_eq!(model.rejection_serial(), 0);
        assert!(model.apply(rejection.clone()));
        assert_eq!(model.rejection_serial(), 1);
        assert!(model.apply(rejection));
        assert_eq!(model.rejection_serial(), 2);
    }

    #[test]
    fn room_closed_event_is_remembered_by_the_client_model() {
        let mut model = ClientModel::new(RoomId(7));
        assert!(!model.room_closed());
        assert!(model.apply(ServerMessage {
            protocol_version: PROTOCOL_VERSION,
            room_id: RoomId(7),
            revision: Revision(1),
            in_reply_to: None,
            event: ServerEvent::RoomClosed,
        }));
        assert!(model.room_closed());
    }

    #[test]
    fn named_leave_notice_and_local_leave_are_remembered_separately() {
        let mut model = ClientModel::new(RoomId(7));
        assert!(model.apply(ServerMessage {
            protocol_version: PROTOCOL_VERSION,
            room_id: RoomId(7),
            revision: Revision(1),
            in_reply_to: None,
            event: ServerEvent::PlayerLeft {
                name: "小明".to_owned(),
            },
        }));
        assert_eq!(model.notice_serial(), 1);
        assert_eq!(model.last_notice(), Some("小明退出了游戏"));
        assert!(!model.left_room());

        assert!(model.apply(ServerMessage {
            protocol_version: PROTOCOL_VERSION,
            room_id: RoomId(7),
            revision: Revision(2),
            in_reply_to: None,
            event: ServerEvent::LeftRoom,
        }));
        assert!(model.left_room());
        assert!(!model.room_closed());
    }

    #[test]
    fn uno_challenge_and_report_events_create_readable_notices() {
        let challenge = UnoEvent::ChallengeResolved {
            challenger: PlayerId(1),
            offender: PlayerId(0),
            result: leocard_uno::ChallengeResult::Successful,
            penalized: PlayerId(0),
            count: 8,
        };
        assert_eq!(
            uno_event_notice(None, &challenge).as_deref(),
            Some("玩家 2 质疑成功，玩家 1 摸 8 张")
        );
        let report = UnoEvent::UnoReported {
            reporter: PlayerId(0),
            target: PlayerId(1),
        };
        assert_eq!(
            uno_event_notice(None, &report).as_deref(),
            Some("玩家 1 检举了 玩家 2，罚摸 2 张")
        );
        let skip = UnoEvent::SkipResolved {
            player: PlayerId(1),
            remaining: 2,
            drew_card: true,
        };
        assert_eq!(uno_event_notice(None, &skip), None);
    }

    #[test]
    fn play_effect_events_advance_an_independent_animation_serial() {
        let mut model = ClientModel::new(RoomId(7));
        let play = PublicPlay {
            kind: leocard_qigui523::PlayKind::Straight { card_count: 3 },
            cards: vec![
                Card::suited(
                    0,
                    leocard_qigui523::Suit::Diamond,
                    leocard_qigui523::Rank::Four,
                ),
                Card::suited(
                    0,
                    leocard_qigui523::Suit::Diamond,
                    leocard_qigui523::Rank::Six,
                ),
                Card::suited(
                    0,
                    leocard_qigui523::Suit::Diamond,
                    leocard_qigui523::Rank::Eight,
                ),
            ],
        };
        assert!(model.apply(ServerMessage {
            protocol_version: PROTOCOL_VERSION,
            room_id: RoomId(7),
            revision: Revision(1),
            in_reply_to: None,
            event: ServerEvent::GameEvent(GameEvent::QiGui523(QiGui523Event::PlayEffect {
                player: PlayerId(2),
                play: play.clone(),
            })),
        }));

        assert_eq!(model.play_effect_serial(), 1);
        assert_eq!(model.last_play_effect(), Some(&(PlayerId(2), play)));
    }

    #[test]
    fn player_interactions_are_queued_in_receive_order_and_drained_once() {
        let mut model = ClientModel::new(RoomId(7));
        let flower = PlayerInteraction {
            source: PlayerId(0),
            target: PlayerId(2),
            kind: PlayerInteractionKind::Flower,
            seed: 11,
        };
        let shoe = PlayerInteraction {
            source: PlayerId(1),
            target: PlayerId(0),
            kind: PlayerInteractionKind::Shoe,
            seed: 22,
        };
        for interaction in [flower, shoe] {
            assert!(model.apply(ServerMessage {
                protocol_version: PROTOCOL_VERSION,
                room_id: RoomId(7),
                revision: Revision(1),
                in_reply_to: None,
                event: ServerEvent::PlayerInteraction(interaction),
            }));
        }

        assert_eq!(model.take_player_interactions(), vec![flower, shoe]);
        assert!(model.take_player_interactions().is_empty());
    }

    #[test]
    fn chat_messages_are_queued_in_receive_order_and_drained_once() {
        let mut model = ClientModel::new(RoomId(7));
        let text = ChatMessage {
            source: PlayerId(0),
            content: ChatContent::Text("你好".to_owned()),
        };
        let voice = ChatMessage {
            source: PlayerId(2),
            content: ChatContent::QuickVoice(3),
        };
        for chat in [text.clone(), voice.clone()] {
            assert!(model.apply(ServerMessage {
                protocol_version: PROTOCOL_VERSION,
                room_id: RoomId(7),
                revision: Revision(1),
                in_reply_to: None,
                event: ServerEvent::ChatMessage(chat),
            }));
        }

        assert_eq!(model.take_chat_messages(), vec![text, voice]);
        assert!(model.take_chat_messages().is_empty());
    }

    fn unused_local_port() -> u16 {
        StdTcpListener::bind(("127.0.0.1", 0))
            .unwrap()
            .local_addr()
            .unwrap()
            .port()
    }

    fn wait_for(client: &mut TcpGameClient, predicate: impl Fn(&ClientModel) -> bool) {
        let deadline = Instant::now() + Duration::from_secs(3);
        while Instant::now() < deadline {
            client.poll();
            if predicate(client.model()) {
                return;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        panic!("timed out; network state is {:?}", client.state());
    }

    #[test]
    fn real_tcp_host_and_two_joiners_reach_the_same_three_player_game() {
        let port = unused_local_port();
        let rules = RuleSet {
            player_count: 3,
            ..RuleSet::default()
        };
        let mut host = TcpGameClient::host("房主", port, rules).unwrap();
        wait_for(&mut host, |model| model.lobby().is_some());

        let address = format!("127.0.0.1:{port}");
        let mut guest_a = TcpGameClient::join("甲", &address).unwrap();
        let mut guest_b = TcpGameClient::join("乙", &address).unwrap();
        wait_for(&mut host, |model| {
            model.lobby().is_some_and(|lobby| lobby.players.len() == 3)
        });
        wait_for(&mut guest_a, |model| {
            model.lobby().is_some_and(|lobby| lobby.players.len() == 3)
        });
        wait_for(&mut guest_b, |model| {
            model.lobby().is_some_and(|lobby| lobby.players.len() == 3)
        });

        assert_eq!(host.model().you(), Some(PlayerId(0)));
        assert_eq!(guest_a.model().room_id(), NETWORK_ROOM_ID);
        assert_eq!(guest_b.model().room_id(), NETWORK_ROOM_ID);
        assert_eq!(host.model().host_port(), Some(port));
        assert_eq!(guest_a.model().host_port(), Some(port));
        assert_eq!(guest_b.model().host_port(), Some(port));

        host.send(ClientCommand::SelectSeat { seat: SeatId(0) });
        guest_a.send(ClientCommand::SelectSeat { seat: SeatId(2) });
        guest_b.send(ClientCommand::SelectSeat { seat: SeatId(5) });
        host.send(ClientCommand::SetReady { ready: true });
        guest_a.send(ClientCommand::SetReady { ready: true });
        guest_b.send(ClientCommand::SetReady { ready: true });
        wait_for(&mut host, |model| {
            model
                .lobby()
                .is_some_and(|lobby| lobby.players.iter().all(|player| player.ready))
        });
        host.send(ClientCommand::StartGame);
        wait_for(&mut host, |model| model.qigui523_game().is_some());
        wait_for(&mut guest_a, |model| model.qigui523_game().is_some());
        wait_for(&mut guest_b, |model| model.qigui523_game().is_some());
        assert_eq!(guest_a.model().qigui523_game().unwrap().host_port, port);
        assert!(host.model().qigui523_game().unwrap().your_hand.len() == rules.hand_size.into());
    }

    #[test]
    fn host_close_room_event_stops_guests_without_reconnecting() {
        let port = unused_local_port();
        let mut host = TcpGameClient::host("房主", port, RuleSet::default()).unwrap();
        wait_for(&mut host, |model| model.lobby().is_some());

        let mut guest = TcpGameClient::join("访客", &format!("127.0.0.1:{port}")).unwrap();
        wait_for(&mut guest, |model| model.lobby().is_some());
        wait_for(&mut host, |model| {
            model.lobby().is_some_and(|lobby| lobby.players.len() == 2)
        });

        assert!(host.send(ClientCommand::CloseRoom));
        wait_for(&mut host, ClientModel::room_closed);
        wait_for(&mut guest, ClientModel::room_closed);

        assert!(!matches!(host.state(), NetworkState::Reconnecting(_)));
        assert!(!matches!(guest.state(), NetworkState::Reconnecting(_)));
    }
}
