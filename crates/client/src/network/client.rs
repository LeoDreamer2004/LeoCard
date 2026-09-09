use super::NETWORK_ROOM_ID;
use super::state::{
    COMMAND_QUEUE, CommandSender, EventQueue, LocalPlayerConnection, NetworkEvent, NetworkLaunch,
    NetworkStartError, NetworkState, normalize_server_address, push_event,
};
use super::worker::run_network;
use crate::ClientModel;
use leocard_host::{GameSetup, HostSession};
use leocard_protocol::{
    ChatMessage, ClientCommand, GameRules, PlayerInteraction, ReconnectToken, ServerEvent,
};
use std::collections::VecDeque;
use std::io;
use std::sync::{Arc, Mutex};
use std::thread;

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
    pub fn host_with_profile(
        port: u16,
        rules: GameRules,
        player: LocalPlayerConnection,
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
        let events = Arc::new(Mutex::new(VecDeque::new()));
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

    pub fn model_mut(&mut self) -> &mut ClientModel {
        &mut self.model
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
