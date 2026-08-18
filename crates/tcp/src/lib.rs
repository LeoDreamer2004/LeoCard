//! Tokio TCP 传输。业务判定仍全部由 `leocard-host` 完成。

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::io;
use std::net::SocketAddr;
use std::time::Duration;

use leocard_host::{ConnectionId, Delivery, HostSession};
use leocard_protocol::{
    ClientMessage, FrameError, MAX_FRAME_PAYLOAD, ServerEvent, ServerMessage, decode_frame,
    encode_frame,
};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;

const CONNECTION_QUEUE: usize = 64;
const INBOUND_QUEUE: usize = 256;
const MAX_CONNECTIONS: usize = 16;
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(3);
const CONNECTION_IDLE_TIMEOUT: Duration = Duration::from_secs(15);
const FRAME_HEADER_TIMEOUT: Duration = Duration::from_secs(5);
const FRAME_BODY_TIMEOUT: Duration = Duration::from_secs(10);
const WRITE_TIMEOUT: Duration = Duration::from_secs(5);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const TURN_TIMER_INTERVAL: Duration = Duration::from_millis(100);

#[derive(Debug)]
pub enum TcpError {
    Io(io::Error),
    Frame(FrameError),
    ConnectionClosed,
    WriterClosed,
    Timeout(&'static str),
    ServerTask(String),
}

impl fmt::Display for TcpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "TCP I/O error: {error}"),
            Self::Frame(error) => error.fmt(f),
            Self::ConnectionClosed => f.write_str("TCP connection closed"),
            Self::WriterClosed => f.write_str("TCP writer task closed"),
            Self::Timeout(operation) => write!(f, "TCP {operation} timed out"),
            Self::ServerTask(error) => write!(f, "TCP server task failed: {error}"),
        }
    }
}

impl std::error::Error for TcpError {}

impl From<io::Error> for TcpError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<FrameError> for TcpError {
    fn from(value: FrameError) -> Self {
        Self::Frame(value)
    }
}

pub async fn write_message<W, T>(writer: &mut W, message: &T) -> Result<(), TcpError>
where
    W: AsyncWrite + Unpin,
    T: serde::Serialize,
{
    let frame = encode_frame(message)?;
    tokio::time::timeout(WRITE_TIMEOUT, writer.write_all(&frame))
        .await
        .map_err(|_| TcpError::Timeout("write"))??;
    Ok(())
}

pub async fn read_message<R, T>(reader: &mut R) -> Result<T, TcpError>
where
    R: AsyncRead + Unpin,
    T: for<'de> serde::Deserialize<'de>,
{
    let mut prefix = [0_u8; 4];
    match tokio::time::timeout(FRAME_HEADER_TIMEOUT, reader.read_exact(&mut prefix)).await {
        Err(_) => return Err(TcpError::Timeout("frame header read")),
        Ok(Ok(_)) => {}
        Ok(Err(error)) if error.kind() == io::ErrorKind::UnexpectedEof => {
            return Err(TcpError::ConnectionClosed);
        }
        Ok(Err(error)) => return Err(TcpError::Io(error)),
    }
    let payload_len = u32::from_be_bytes(prefix) as usize;
    if payload_len > MAX_FRAME_PAYLOAD {
        return Err(TcpError::Frame(FrameError::PayloadTooLarge {
            actual: payload_len,
            maximum: MAX_FRAME_PAYLOAD,
        }));
    }
    let mut frame = Vec::with_capacity(payload_len + 4);
    frame.extend_from_slice(&prefix);
    frame.resize(payload_len + 4, 0);
    tokio::time::timeout(FRAME_BODY_TIMEOUT, reader.read_exact(&mut frame[4..]))
        .await
        .map_err(|_| TcpError::Timeout("frame body read"))??;
    Ok(decode_frame(&frame)?)
}

pub struct TcpClient {
    reader: OwnedReadHalf,
    writer: OwnedWriteHalf,
}

/// TCP 客户端的只读半边。拆分后可与命令发送任务并行等待服务端广播。
pub struct TcpClientReader(OwnedReadHalf);

/// TCP 客户端的只写半边。
pub struct TcpClientWriter(OwnedWriteHalf);

impl TcpClient {
    pub async fn connect(address: impl ToSocketAddrs) -> Result<Self, TcpError> {
        let stream = tokio::time::timeout(CONNECT_TIMEOUT, TcpStream::connect(address))
            .await
            .map_err(|_| TcpError::Timeout("connect"))??;
        stream.set_nodelay(true)?;
        let (reader, writer) = stream.into_split();
        Ok(Self { reader, writer })
    }

    pub async fn send(&mut self, message: &ClientMessage) -> Result<(), TcpError> {
        write_message(&mut self.writer, message).await
    }

    pub async fn receive(&mut self) -> Result<ServerMessage, TcpError> {
        read_message(&mut self.reader).await
    }

    pub fn into_split(self) -> (TcpClientReader, TcpClientWriter) {
        (TcpClientReader(self.reader), TcpClientWriter(self.writer))
    }
}

impl TcpClientReader {
    pub async fn receive(&mut self) -> Result<ServerMessage, TcpError> {
        read_message(&mut self.0).await
    }
}

impl TcpClientWriter {
    pub async fn send(&mut self, message: &ClientMessage) -> Result<(), TcpError> {
        write_message(&mut self.0, message).await
    }
}

pub struct TcpServerHandle {
    local_addr: SocketAddr,
    shutdown: Option<oneshot::Sender<()>>,
    task: JoinHandle<Result<(), TcpError>>,
}

impl TcpServerHandle {
    pub async fn bind(address: impl ToSocketAddrs, session: HostSession) -> Result<Self, TcpError> {
        let listener = TcpListener::bind(address).await?;
        let local_addr = listener.local_addr()?;
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        let task = tokio::spawn(run_server(listener, session, shutdown_rx));
        Ok(Self {
            local_addr,
            shutdown: Some(shutdown_tx),
            task,
        })
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }

    pub async fn shutdown(mut self) -> Result<(), TcpError> {
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }
        self.task
            .await
            .map_err(|error| TcpError::ServerTask(error.to_string()))?
    }
}

enum Inbound {
    Message(ConnectionId, ClientMessage),
    Disconnected(ConnectionId),
}

type ConnectionTasks = (JoinHandle<()>, JoinHandle<()>);

async fn run_server(
    listener: TcpListener,
    mut session: HostSession,
    mut shutdown: oneshot::Receiver<()>,
) -> Result<(), TcpError> {
    let (inbound_tx, mut inbound_rx) = mpsc::channel(INBOUND_QUEUE);
    let mut writers: HashMap<ConnectionId, mpsc::Sender<ServerMessage>> = HashMap::new();
    let mut tasks: HashMap<ConnectionId, ConnectionTasks> = HashMap::new();
    let mut authenticated = HashSet::new();
    let mut last_seen = HashMap::new();
    let mut next_connection = 1_u64;
    let mut heartbeat = tokio::time::interval(HEARTBEAT_INTERVAL);
    heartbeat.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let mut idle_check = tokio::time::interval(CONNECTION_IDLE_TIMEOUT / 3);
    idle_check.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let mut turn_timer = tokio::time::interval(TURN_TIMER_INTERVAL);
    turn_timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let mut last_timer_tick = tokio::time::Instant::now();
    let mut flush_writers = false;

    loop {
        tokio::select! {
            biased;
            _ = &mut shutdown => break,
            accepted = listener.accept() => {
                let (mut stream, _) = accepted?;
                if writers.len() >= MAX_CONNECTIONS {
                    let _ = stream.shutdown().await;
                    continue;
                }
                stream.set_nodelay(true)?;
                let connection = ConnectionId(next_connection);
                next_connection += 1;
                let (reader, writer) = stream.into_split();
                let (writer_tx, writer_rx) = mpsc::channel(CONNECTION_QUEUE);
                writers.insert(connection, writer_tx);
                last_seen.insert(connection, tokio::time::Instant::now());
                let reader_task = tokio::spawn(reader_loop(connection, reader, inbound_tx.clone()));
                let writer_task = tokio::spawn(writer_loop(
                    connection,
                    writer,
                    writer_rx,
                    inbound_tx.clone(),
                ));
                tasks.insert(connection, (reader_task, writer_task));
            }
            inbound = inbound_rx.recv() => {
                let Some(inbound) = inbound else { break };
                let connection = match &inbound {
                    Inbound::Message(connection, _) | Inbound::Disconnected(connection) => *connection,
                };
                if matches!(&inbound, Inbound::Message(_, _)) {
                    last_seen.insert(connection, tokio::time::Instant::now());
                }
                let deliveries = match inbound {
                    Inbound::Message(connection, message) => {
                        let deliveries = session.handle(connection, message);
                        if deliveries.iter().any(|delivery| {
                            delivery.recipient == connection
                                && matches!(delivery.message.event, ServerEvent::Joined { .. })
                        }) {
                            authenticated.insert(connection);
                        }
                        deliveries
                    }
                    Inbound::Disconnected(connection) => {
                        retire_connection(
                            connection,
                            &mut writers,
                            &mut last_seen,
                            &mut tasks,
                            &mut authenticated,
                        );
                        session.disconnect(connection)
                    }
                };
                let replaced = authenticated
                    .iter()
                    .copied()
                    .filter(|candidate| {
                        *candidate != connection && !session.is_current_connection(*candidate)
                    })
                    .collect::<Vec<_>>();
                for replaced in replaced {
                    retire_connection(
                        replaced,
                        &mut writers,
                        &mut last_seen,
                        &mut tasks,
                        &mut authenticated,
                    );
                }
                dispatch_deliveries(
                    &mut session,
                    &mut writers,
                    &mut last_seen,
                    &mut tasks,
                    &mut authenticated,
                    deliveries,
                );
                if session.is_closed() {
                    flush_writers = true;
                    break;
                }
            }
            _ = heartbeat.tick() => {
                let deliveries = session.heartbeat();
                dispatch_deliveries(
                    &mut session,
                    &mut writers,
                    &mut last_seen,
                    &mut tasks,
                    &mut authenticated,
                    deliveries,
                );
            }
            _ = turn_timer.tick() => {
                let now = tokio::time::Instant::now();
                let elapsed = now.duration_since(last_timer_tick);
                last_timer_tick = now;
                let deliveries = session.advance_time(elapsed);
                dispatch_deliveries(
                    &mut session,
                    &mut writers,
                    &mut last_seen,
                    &mut tasks,
                    &mut authenticated,
                    deliveries,
                );
                if session.is_closed() {
                    flush_writers = true;
                    break;
                }
            }
            _ = idle_check.tick() => {
                let now = tokio::time::Instant::now();
                let stale = last_seen
                    .iter()
                    .filter_map(|(connection, seen)| {
                        (now.duration_since(*seen) >= CONNECTION_IDLE_TIMEOUT).then_some(*connection)
                })
                    .collect::<Vec<_>>();
                for connection in stale {
                    retire_connection(
                        connection,
                        &mut writers,
                        &mut last_seen,
                        &mut tasks,
                        &mut authenticated,
                    );
                    let deliveries = session.disconnect(connection);
                    dispatch_deliveries(
                        &mut session,
                        &mut writers,
                        &mut last_seen,
                        &mut tasks,
                        &mut authenticated,
                        deliveries,
                    );
                }
            }
        }
    }
    if flush_writers {
        writers.clear();
        for (reader, writer) in tasks.into_values() {
            reader.abort();
            let _ = writer.await;
        }
    } else {
        for (reader, writer) in tasks.into_values() {
            reader.abort();
            writer.abort();
        }
    }
    Ok(())
}

async fn reader_loop(
    connection: ConnectionId,
    mut reader: OwnedReadHalf,
    inbound: mpsc::Sender<Inbound>,
) {
    loop {
        match read_message(&mut reader).await {
            Ok(message) => {
                if inbound
                    .send(Inbound::Message(connection, message))
                    .await
                    .is_err()
                {
                    return;
                }
            }
            Err(_) => {
                let _ = inbound.send(Inbound::Disconnected(connection)).await;
                return;
            }
        }
    }
}

async fn writer_loop(
    connection: ConnectionId,
    mut writer: OwnedWriteHalf,
    mut messages: mpsc::Receiver<ServerMessage>,
    inbound: mpsc::Sender<Inbound>,
) {
    while let Some(message) = messages.recv().await {
        if write_message(&mut writer, &message).await.is_err() {
            let _ = inbound.send(Inbound::Disconnected(connection)).await;
            return;
        }
    }
}

fn dispatch_deliveries(
    session: &mut HostSession,
    writers: &mut HashMap<ConnectionId, mpsc::Sender<ServerMessage>>,
    last_seen: &mut HashMap<ConnectionId, tokio::time::Instant>,
    tasks: &mut HashMap<ConnectionId, ConnectionTasks>,
    authenticated: &mut HashSet<ConnectionId>,
    deliveries: Vec<Delivery>,
) {
    let mut pending = deliveries;
    while !pending.is_empty() {
        let failed_connections = route_deliveries(writers, pending);
        if failed_connections.is_empty() {
            break;
        }
        pending = failed_connections
            .into_iter()
            .flat_map(|connection| {
                retire_connection(connection, writers, last_seen, tasks, authenticated);
                session.disconnect(connection)
            })
            .collect();
    }
}

fn route_deliveries(
    writers: &mut HashMap<ConnectionId, mpsc::Sender<ServerMessage>>,
    deliveries: Vec<Delivery>,
) -> Vec<ConnectionId> {
    let mut failed_connections = Vec::new();
    for delivery in deliveries {
        let failed = match writers.get(&delivery.recipient) {
            Some(writer) => writer.try_send(delivery.message).is_err(),
            None => false,
        };
        if failed {
            writers.remove(&delivery.recipient);
            if !failed_connections.contains(&delivery.recipient) {
                failed_connections.push(delivery.recipient);
            }
        }
    }
    failed_connections
}

fn retire_connection(
    connection: ConnectionId,
    writers: &mut HashMap<ConnectionId, mpsc::Sender<ServerMessage>>,
    last_seen: &mut HashMap<ConnectionId, tokio::time::Instant>,
    tasks: &mut HashMap<ConnectionId, ConnectionTasks>,
    authenticated: &mut HashSet<ConnectionId>,
) {
    writers.remove(&connection);
    last_seen.remove(&connection);
    authenticated.remove(&connection);
    if let Some((reader, writer)) = tasks.remove(&connection) {
        reader.abort();
        writer.abort();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};
    use leocard_protocol::{
        ClientCommand, GameSnapshot, PlayerId, ProfileId, QiGui523Snapshot, ReconnectToken,
        RequestId, RoomId, SeatId, ServerEvent, ShengjiSnapshot, TexasHoldemSnapshot, UnoSnapshot,
        join_identity_payload,
    };
    use leocard_qigui523::{RuleSet, build_deck};
    use leocard_shengji::{RuleSet as ShengjiRuleSet, build_deck as build_shengji_deck};
    use leocard_texas_holdem::{RuleSet as TexasHoldemRuleSet, build_deck as build_texas_deck};
    use leocard_uno::{RuleSet as UnoRuleSet, build_deck as build_uno_deck};
    use tokio::time::{Duration, timeout};

    async fn receive_joined(client: &mut TcpClient) -> PlayerId {
        timeout(Duration::from_secs(2), async {
            loop {
                if let ServerEvent::Joined { you } = client.receive().await.unwrap().event {
                    return you;
                }
            }
        })
        .await
        .expect("server should confirm the join")
    }

    async fn receive_ready(client: &mut TcpClient, you: PlayerId) {
        timeout(Duration::from_secs(2), async {
            loop {
                if let ServerEvent::LobbySnapshot(snapshot) = client.receive().await.unwrap().event
                    && snapshot
                        .players
                        .iter()
                        .any(|player| player.id == you && player.ready)
                {
                    return;
                }
            }
        })
        .await
        .expect("server should publish the ready state");
    }

    async fn receive_game(client: &mut TcpClient) -> QiGui523Snapshot {
        timeout(Duration::from_secs(2), async {
            loop {
                if let ServerEvent::GameSnapshot(snapshot) = client.receive().await.unwrap().event {
                    return snapshot.into_qigui523().expect("七鬼五二三快照");
                }
            }
        })
        .await
        .expect("server should send a game snapshot")
    }

    async fn receive_texas_game(client: &mut TcpClient) -> TexasHoldemSnapshot {
        timeout(Duration::from_secs(2), async {
            loop {
                if let ServerEvent::GameSnapshot(GameSnapshot::TexasHoldem(snapshot)) =
                    client.receive().await.unwrap().event
                {
                    return snapshot;
                }
            }
        })
        .await
        .expect("server should send a Texas Hold'em snapshot")
    }

    async fn receive_shengji_dealt_card(client: &mut TcpClient) -> ShengjiSnapshot {
        timeout(Duration::from_secs(3), async {
            loop {
                if let ServerEvent::GameSnapshot(GameSnapshot::Shengji(snapshot)) =
                    client.receive().await.unwrap().event
                    && !snapshot.your_hand.is_empty()
                {
                    return snapshot;
                }
            }
        })
        .await
        .expect("slow deal should reach every Shengji player")
    }

    async fn receive_uno_game(client: &mut TcpClient) -> UnoSnapshot {
        timeout(Duration::from_secs(2), async {
            loop {
                if let ServerEvent::GameSnapshot(GameSnapshot::Uno(snapshot)) =
                    client.receive().await.unwrap().event
                {
                    return snapshot;
                }
            }
        })
        .await
        .expect("server should send an UNO snapshot")
    }

    #[tokio::test]
    async fn three_clients_join_ready_and_start_over_real_tcp() {
        let room = RoomId(523);
        let rules = RuleSet {
            player_count: 3,
            ..RuleSet::default()
        };
        let session = HostSession::qigui523(room, 52300, rules, build_deck(1)).unwrap();
        let server = TcpServerHandle::bind("127.0.0.1:0", session).await.unwrap();
        let address = server.local_addr();
        let mut clients = Vec::new();
        let mut player_ids = Vec::new();

        for index in 0..3_u64 {
            let mut client = TcpClient::connect(address).await.unwrap();
            let name = format!("P{index}");
            let token = ReconnectToken(index);
            let mut secret = [0; 32];
            secret[..8].copy_from_slice(&index.to_be_bytes());
            secret[8] = 1;
            let key = SigningKey::from_bytes(&secret);
            let game_profiles = leocard_protocol::PlayerGameProfiles::default();
            let signature = key
                .sign(&join_identity_payload(
                    room,
                    token,
                    &name,
                    0,
                    0,
                    &game_profiles,
                ))
                .to_bytes()
                .to_vec();
            client
                .send(&ClientMessage::new(
                    room,
                    RequestId(1),
                    ClientCommand::Join {
                        name,
                        reconnect_token: token,
                        profile_id: ProfileId(key.verifying_key().to_bytes()),
                        reference_points: 0,
                        completed_games: 0,
                        game_profiles,
                        identity_signature: signature,
                    },
                ))
                .await
                .unwrap();
            let player_id = receive_joined(&mut client).await;
            client
                .send(&ClientMessage::new(
                    room,
                    RequestId(2),
                    ClientCommand::SelectSeat {
                        seat: SeatId(index as u8),
                    },
                ))
                .await
                .unwrap();
            client
                .send(&ClientMessage::new(
                    room,
                    RequestId(3),
                    ClientCommand::SetReady { ready: true },
                ))
                .await
                .unwrap();
            receive_ready(&mut client, player_id).await;
            clients.push(client);
            player_ids.push(player_id);
        }

        let host_index = player_ids
            .iter()
            .position(|player| *player == PlayerId(0))
            .expect("one connected client should be the host");
        clients[host_index]
            .send(&ClientMessage::new(
                room,
                RequestId(4),
                ClientCommand::StartGame,
            ))
            .await
            .unwrap();

        for (expected_player, client) in player_ids.into_iter().zip(&mut clients) {
            let snapshot = receive_game(client).await;
            assert_eq!(snapshot.you, expected_player);
            assert_eq!(snapshot.your_hand.len(), 5);
            assert!(snapshot.players.iter().all(|player| player.hand_len == 5));
        }

        server.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn texas_room_uses_the_same_tcp_lobby_and_private_snapshots() {
        let room = RoomId(9527);
        let rules = TexasHoldemRuleSet::default();
        let session =
            HostSession::texas_holdem(room, 52301, rules, build_texas_deck(false)).unwrap();
        let server = TcpServerHandle::bind("127.0.0.1:0", session).await.unwrap();
        let address = server.local_addr();
        let mut clients = Vec::new();
        let mut player_ids = Vec::new();

        for index in 0..3_u64 {
            let mut client = TcpClient::connect(address).await.unwrap();
            let name = format!("T{index}");
            let token = ReconnectToken(index + 100);
            let mut secret = [0; 32];
            secret[..8].copy_from_slice(&(index + 100).to_be_bytes());
            secret[8] = 2;
            let key = SigningKey::from_bytes(&secret);
            let game_profiles = leocard_protocol::PlayerGameProfiles::default();
            let signature = key
                .sign(&join_identity_payload(
                    room,
                    token,
                    &name,
                    0,
                    0,
                    &game_profiles,
                ))
                .to_bytes()
                .to_vec();
            client
                .send(&ClientMessage::new(
                    room,
                    RequestId(1),
                    ClientCommand::Join {
                        name,
                        reconnect_token: token,
                        profile_id: ProfileId(key.verifying_key().to_bytes()),
                        reference_points: 0,
                        completed_games: 0,
                        game_profiles,
                        identity_signature: signature,
                    },
                ))
                .await
                .unwrap();
            let player = receive_joined(&mut client).await;
            client
                .send(&ClientMessage::new(
                    room,
                    RequestId(2),
                    ClientCommand::SelectSeat {
                        seat: SeatId(index as u8),
                    },
                ))
                .await
                .unwrap();
            client
                .send(&ClientMessage::new(
                    room,
                    RequestId(3),
                    ClientCommand::SetReady { ready: true },
                ))
                .await
                .unwrap();
            receive_ready(&mut client, player).await;
            clients.push(client);
            player_ids.push(player);
        }

        let host = player_ids.iter().position(|id| *id == PlayerId(0)).unwrap();
        clients[host]
            .send(&ClientMessage::new(
                room,
                RequestId(4),
                ClientCommand::StartGame,
            ))
            .await
            .unwrap();

        let mut private_hands = Vec::new();
        for (expected, client) in player_ids.into_iter().zip(&mut clients) {
            let snapshot = receive_texas_game(client).await;
            assert_eq!(snapshot.you, expected);
            assert_eq!(snapshot.your_hole_cards.len(), 2);
            assert!(snapshot.revealed_hands.is_empty());
            assert_eq!(snapshot.players.len(), 3);
            private_hands.push(snapshot.your_hole_cards);
        }
        assert_ne!(private_hands[0], private_hands[1]);
        assert_ne!(private_hands[1], private_hands[2]);

        server.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn shengji_room_slow_deals_private_hands_over_the_shared_tcp_transport() {
        let room = RoomId(8080);
        let session =
            HostSession::shengji(room, 52302, ShengjiRuleSet::default(), build_shengji_deck())
                .unwrap();
        let server = TcpServerHandle::bind("127.0.0.1:0", session).await.unwrap();
        let address = server.local_addr();
        let mut clients = Vec::new();
        let mut player_ids = Vec::new();

        for index in 0..4_u64 {
            let mut client = TcpClient::connect(address).await.unwrap();
            let name = format!("S{index}");
            let token = ReconnectToken(index + 200);
            let mut secret = [0; 32];
            secret[..8].copy_from_slice(&(index + 200).to_be_bytes());
            secret[8] = 3;
            let key = SigningKey::from_bytes(&secret);
            let game_profiles = leocard_protocol::PlayerGameProfiles::default();
            let signature = key
                .sign(&join_identity_payload(
                    room,
                    token,
                    &name,
                    0,
                    0,
                    &game_profiles,
                ))
                .to_bytes()
                .to_vec();
            client
                .send(&ClientMessage::new(
                    room,
                    RequestId(1),
                    ClientCommand::Join {
                        name,
                        reconnect_token: token,
                        profile_id: ProfileId(key.verifying_key().to_bytes()),
                        reference_points: 0,
                        completed_games: 0,
                        game_profiles,
                        identity_signature: signature,
                    },
                ))
                .await
                .unwrap();
            let player = receive_joined(&mut client).await;
            client
                .send(&ClientMessage::new(
                    room,
                    RequestId(2),
                    ClientCommand::SelectSeat {
                        seat: SeatId(index as u8),
                    },
                ))
                .await
                .unwrap();
            client
                .send(&ClientMessage::new(
                    room,
                    RequestId(3),
                    ClientCommand::SetReady { ready: true },
                ))
                .await
                .unwrap();
            receive_ready(&mut client, player).await;
            clients.push(client);
            player_ids.push(player);
        }

        let host = player_ids.iter().position(|id| *id == PlayerId(0)).unwrap();
        clients[host]
            .send(&ClientMessage::new(
                room,
                RequestId(4),
                ClientCommand::StartGame,
            ))
            .await
            .unwrap();

        for (expected, client) in player_ids.into_iter().zip(&mut clients) {
            let snapshot = receive_shengji_dealt_card(client).await;
            assert_eq!(snapshot.you, expected);
            assert_eq!(snapshot.your_hand.len(), 1);
            assert_eq!(snapshot.players.len(), 4);
            assert_eq!(
                snapshot
                    .players
                    .iter()
                    .find(|player| player.id == expected)
                    .map(|player| player.hand_len),
                Some(1)
            );
            let dealt = snapshot
                .players
                .iter()
                .map(|player| player.hand_len)
                .sum::<u8>();
            // 四个客户端是逐个读取的，定时发牌可在相邻读取之间继续推进；这里
            // 验证收到的是该玩家第一轮的私有手牌，而不把全桌时钟钉死在某一拍。
            assert!((expected.0 + 1..=expected.0 + 4).contains(&dealt));
        }

        server.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn uno_room_uses_the_shared_tcp_transport_and_private_hands() {
        let room = RoomId(1080);
        let rules = UnoRuleSet::default();
        let session = HostSession::uno(room, 52303, rules, build_uno_deck()).unwrap();
        let server = TcpServerHandle::bind("127.0.0.1:0", session).await.unwrap();
        let address = server.local_addr();
        let mut clients = Vec::new();
        let mut player_ids = Vec::new();

        for index in 0..2_u64 {
            let mut client = TcpClient::connect(address).await.unwrap();
            let name = format!("U{index}");
            let token = ReconnectToken(index + 300);
            let mut secret = [0; 32];
            secret[..8].copy_from_slice(&(index + 300).to_be_bytes());
            secret[8] = 4;
            let key = SigningKey::from_bytes(&secret);
            let game_profiles = leocard_protocol::PlayerGameProfiles::default();
            let signature = key
                .sign(&join_identity_payload(
                    room,
                    token,
                    &name,
                    0,
                    0,
                    &game_profiles,
                ))
                .to_bytes()
                .to_vec();
            client
                .send(&ClientMessage::new(
                    room,
                    RequestId(1),
                    ClientCommand::Join {
                        name,
                        reconnect_token: token,
                        profile_id: ProfileId(key.verifying_key().to_bytes()),
                        reference_points: 0,
                        completed_games: 0,
                        game_profiles,
                        identity_signature: signature,
                    },
                ))
                .await
                .unwrap();
            let player = receive_joined(&mut client).await;
            client
                .send(&ClientMessage::new(
                    room,
                    RequestId(2),
                    ClientCommand::SelectSeat {
                        seat: SeatId(index as u8),
                    },
                ))
                .await
                .unwrap();
            client
                .send(&ClientMessage::new(
                    room,
                    RequestId(3),
                    ClientCommand::SetReady { ready: true },
                ))
                .await
                .unwrap();
            receive_ready(&mut client, player).await;
            clients.push(client);
            player_ids.push(player);
        }

        let host = player_ids.iter().position(|id| *id == PlayerId(0)).unwrap();
        clients[host]
            .send(&ClientMessage::new(
                room,
                RequestId(4),
                ClientCommand::StartGame,
            ))
            .await
            .unwrap();

        let mut hands = Vec::new();
        for (expected, client) in player_ids.into_iter().zip(&mut clients) {
            let snapshot = receive_uno_game(client).await;
            assert_eq!(snapshot.you, expected);
            assert_eq!(snapshot.your_hand.len(), 7);
            assert_eq!(snapshot.players.len(), 2);
            assert!(snapshot.players.iter().all(|player| player.hand_len == 7));
            hands.push(snapshot.your_hand);
        }
        assert!(hands.windows(2).any(|pair| pair[0] != pair[1]));

        server.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn framed_codec_works_when_bytes_arrive_in_fragments() {
        let (mut left, mut right) = tokio::io::duplex(64);
        let message = ClientMessage::new(RoomId(1), RequestId(1), ClientCommand::RequestSnapshot);
        let frame = encode_frame(&message).unwrap();
        let writer = tokio::spawn(async move {
            for chunk in frame.chunks(2) {
                left.write_all(chunk).await.unwrap();
            }
        });
        let decoded: ClientMessage = read_message(&mut right).await.unwrap();
        writer.await.unwrap();
        assert_eq!(decoded, message);
    }
}
