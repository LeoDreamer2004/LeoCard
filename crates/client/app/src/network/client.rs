use super::NETWORK_ROOM_ID;
use super::backend::{WorkerHandle, forget_session, remembered_reconnect_token, spawn_worker};
use super::state::{
    CommandSender, EventQueue, LocalPlayerConnection, NetworkEvent, NetworkLaunch,
    NetworkStartError, NetworkState, normalize_server_address,
};
use crate::ClientModel;
use leocard_game_common::PlayerId;
#[cfg(not(feature = "web"))]
use leocard_host::{GameSetup, HostSession};
use leocard_platform::time::{Duration, Instant};
use leocard_protocol::{
    ChatMessage, ClientCommand, GameCommand, GameRules, PlayerInteraction, ReconnectToken,
    Revision, ServerEvent, ServerMessage,
};
#[cfg(not(feature = "web"))]
use leocard_tcp::ServerTls;
use std::collections::VecDeque;
use std::io;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

const GAME_COMMAND_REPEAT_WINDOW: Duration = Duration::from_millis(500);
#[cfg(not(target_arch = "wasm32"))]
static NEXT_SESSION_ID: AtomicU64 = AtomicU64::new(1);

struct RecentGameCommand {
    command: GameCommand,
    revision: Revision,
    sent_at: Instant,
}

impl RecentGameCommand {
    fn repeats(&self, command: &GameCommand, revision: Revision, now: Instant) -> bool {
        self.revision == revision
            && &self.command == command
            && now.duration_since(self.sent_at) < GAME_COMMAND_REPEAT_WINDOW
    }
}

/// 共用的联机客户端，平台后端负责非阻塞传输和重连。
pub struct GameClient {
    #[cfg(not(target_arch = "wasm32"))]
    session_id: u64,
    model: ClientModel,
    commands: CommandSender,
    events: EventQueue,
    state: NetworkState,
    recent_game_command: Option<RecentGameCommand>,
    reconnect_started: Option<Instant>,
    reconnect_notice: bool,
    _worker: WorkerHandle,
}

impl GameClient {
    #[cfg(feature = "web")]
    pub fn host_with_profile(
        _port: u16,
        _rules: GameRules,
        _player: LocalPlayerConnection,
    ) -> Result<Self, NetworkStartError> {
        Err(NetworkStartError::Unsupported("网页端仅支持加入房间"))
    }

    /// 在所有网卡上监听指定端口，并让房主通过回环 TCP 加入自己的房间。
    #[cfg(not(feature = "web"))]
    pub fn host_with_profile(
        port: u16,
        rules: GameRules,
        player: LocalPlayerConnection,
    ) -> Result<Self, NetworkStartError> {
        Self::spawn_host(port, rules, player, None)
    }

    /// 在原有 TCP/WS 开房端口上同时接受使用指定证书的 WSS 连接。
    #[cfg(not(feature = "web"))]
    pub fn host_with_tls(
        port: u16,
        rules: GameRules,
        player: LocalPlayerConnection,
        tls: ServerTls,
    ) -> Result<Self, NetworkStartError> {
        Self::spawn_host(port, rules, player, Some(tls))
    }

    #[cfg(not(feature = "web"))]
    fn spawn_host(
        port: u16,
        rules: GameRules,
        player: LocalPlayerConnection,
        tls: Option<ServerTls>,
    ) -> Result<Self, NetworkStartError> {
        if port == 0 {
            return Err(NetworkStartError::InvalidPort);
        }
        let session = HostSession::new(NETWORK_ROOM_ID, GameSetup::shuffled(port, rules))
            .map_err(|error| NetworkStartError::Worker(io::Error::other(error)))?;
        Self::spawn(
            player,
            NetworkLaunch::Host {
                port,
                session: Box::new(session),
                tls,
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
        let events = Arc::new(Mutex::new(VecDeque::new()));
        let mut model = ClientModel::new(NETWORK_ROOM_ID);
        let fresh_token = || -> Result<u64, NetworkStartError> {
            loop {
                let token = getrandom::u64().map_err(|error| {
                    NetworkStartError::Worker(io::Error::other(format!(
                        "无法生成重连令牌：{error}"
                    )))
                })?;
                if token != 0 {
                    break Ok(token);
                }
            }
        };
        let reconnect_token = ReconnectToken(
            remembered_reconnect_token(&launch)
                .map(Ok)
                .unwrap_or_else(fresh_token)?,
        );
        let mut command = player.identity.join_command(
            NETWORK_ROOM_ID,
            player.name.trim(),
            reconnect_token,
            player.reference_points,
            player.completed_games,
            player.game_profiles,
        );
        if let ClientCommand::Join(request) = &mut command {
            request.avatar_png = player.avatar_png;
        }
        let join = model.command(command);
        let reconnect_join = join.clone();
        let (command_tx, worker) = spawn_worker(launch, Arc::clone(&events), reconnect_join)?;

        let client = Self {
            #[cfg(not(target_arch = "wasm32"))]
            session_id: NEXT_SESSION_ID.fetch_add(1, Ordering::Relaxed),
            model,
            commands: command_tx,
            events,
            state: NetworkState::Connecting(connecting),
            recent_game_command: None,
            reconnect_started: None,
            reconnect_notice: false,
            _worker: worker,
        };
        Ok(client)
    }

    pub fn model(&self) -> &ClientModel {
        &self.model
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn session_id(&self) -> u64 {
        self.session_id
    }

    pub fn model_mut(&mut self) -> &mut ClientModel {
        &mut self.model
    }

    pub fn state(&self) -> &NetworkState {
        &self.state
    }

    /// Brief outages block commands immediately, but only sustained outages need an overlay.
    pub fn reconnecting_message(&self) -> Option<&str> {
        match &self.state {
            NetworkState::Reconnecting(message) if self.reconnect_notice => Some(message),
            _ => None,
        }
    }

    pub fn send(&mut self, command: ClientCommand) -> bool {
        if matches!(command, ClientCommand::LeaveRoom) {
            forget_session();
        }
        if !matches!(self.state, NetworkState::Connected(_)) {
            return false;
        }
        if matches!(command, ClientCommand::Game(_))
            && self
                .model
                .game_snapshot()
                .is_some_and(|game| self.model.you() != Some(game.you()))
        {
            return false;
        }
        if self.model.is_spectating()
            && !matches!(
                command,
                ClientCommand::RequestSnapshot
                    | ClientCommand::LeaveRoom
                    | ClientCommand::PublishEconomy { .. }
                    | ClientCommand::PublishAchievements { .. }
                    | ClientCommand::Interact { .. }
                    | ClientCommand::SpectatePlayer { .. }
                    | ClientCommand::Chat { .. }
            )
        {
            return false;
        }
        let now = Instant::now();
        let game_command = match &command {
            ClientCommand::Game(game) => Some((game.clone(), self.model.latest_revision())),
            _ => None,
        };
        if game_command.as_ref().is_some_and(|(game, revision)| {
            self.recent_game_command
                .as_ref()
                .is_some_and(|recent| recent.repeats(game, *revision, now))
        }) {
            return false;
        }
        let message = self.model.command(command);
        let sent = self.commands.try_send(message).is_ok();
        if sent && let Some((command, revision)) = game_command {
            self.recent_game_command = Some(RecentGameCommand {
                command,
                revision,
                sent_at: now,
            });
        }
        sent
    }

    pub fn take_player_interactions(&mut self) -> Vec<PlayerInteraction> {
        self.model.take_player_interactions()
    }

    pub fn take_chat_messages(&mut self) -> Vec<ChatMessage> {
        self.model.take_chat_messages()
    }

    /// 把后台线程已经收到的消息应用到模型，返回是否有可见状态变化。
    pub fn poll(&mut self) -> bool {
        self.poll_with_events(|_, _| {})
    }

    /// Route accepted events to application observers without storing feature state
    /// in the transport or client model.
    pub fn poll_with_events(
        &mut self,
        mut observe: impl FnMut(Option<PlayerId>, &ServerMessage),
    ) -> bool {
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
                #[cfg(feature = "web")]
                NetworkEvent::Resync => {
                    if matches!(self.state, NetworkState::Connected(_)) {
                        let message = self.model.command(ClientCommand::RequestSnapshot);
                        let _ = self.commands.try_send(message);
                    }
                }
                NetworkEvent::Connected(address) => {
                    let state = NetworkState::Connected(address);
                    changed |= self.state != state;
                    self.state = state;
                    self.reconnect_started = None;
                }
                NetworkEvent::Reconnecting(message) => {
                    self.reconnect_started.get_or_insert_with(Instant::now);
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
                    changed |= self.model.apply_with_events(*message, &mut observe) && visible;
                }
                NetworkEvent::Failed(error) => {
                    let state = NetworkState::Failed(error);
                    changed |= self.state != state;
                    self.state = state;
                }
            }
        }
        let notice = matches!(self.state, NetworkState::Reconnecting(_))
            && self
                .reconnect_started
                .is_some_and(|started| started.elapsed() >= Duration::from_millis(500));
        changed |= self.reconnect_notice != notice;
        self.reconnect_notice = notice;
        changed
    }
}
