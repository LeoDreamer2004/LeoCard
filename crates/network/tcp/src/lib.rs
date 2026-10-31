//! Tokio TCP 传输。业务判定仍全部由 `leocard-host` 完成。

mod codec;
mod connections;
mod discovery;
mod execution;

pub use codec::{read_message, write_message};
use discovery::LanAnnouncer;
pub use discovery::{LanDiscoverySocket, LanRoomAnnouncement, query_room_info};

use codec::FrameReader;
use connections::{finish_connection, retire_connection};
#[cfg(test)]
mod tests;

use execution::handle_command;
use leocard_host::{ConnectionId, Delivery, HostSession};
use leocard_protocol::{ClientMessage, FrameError, ServerEvent, ServerMessage};
use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::time::Duration;
use std::{fmt, io};
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;

const CONNECTION_QUEUE: usize = 64;
const INBOUND_QUEUE: usize = 256;
const MAX_CONNECTIONS: usize = 16;
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(3);
const CONNECTION_IDLE_TIMEOUT: Duration = Duration::from_secs(15);
const FRAME_HEADER_TIMEOUT: Duration = CONNECTION_IDLE_TIMEOUT;
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

pub struct TcpClient {
    reader: FrameReader<OwnedReadHalf>,
    writer: OwnedWriteHalf,
}

/// TCP 客户端的只读半边。拆分后可与命令发送任务并行等待服务端广播。
pub struct TcpClientReader(FrameReader<OwnedReadHalf>);

/// TCP 客户端的只写半边。
pub struct TcpClientWriter(OwnedWriteHalf);

impl TcpClient {
    pub async fn connect(address: impl ToSocketAddrs) -> Result<Self, TcpError> {
        let stream = tokio::time::timeout(CONNECT_TIMEOUT, TcpStream::connect(address))
            .await
            .map_err(|_| TcpError::Timeout("connect"))??;
        stream.set_nodelay(true)?;
        let (reader, writer) = stream.into_split();
        Ok(Self {
            reader: FrameReader::new(reader),
            writer,
        })
    }

    pub async fn send(&mut self, message: &ClientMessage) -> Result<(), TcpError> {
        write_message(&mut self.writer, message).await
    }

    pub async fn receive(&mut self) -> Result<ServerMessage, TcpError> {
        self.reader.receive().await
    }

    pub fn into_split(self) -> (TcpClientReader, TcpClientWriter) {
        (TcpClientReader(self.reader), TcpClientWriter(self.writer))
    }
}

impl TcpClientReader {
    pub async fn receive(&mut self) -> Result<ServerMessage, TcpError> {
        self.0.receive().await
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
    let announcer = LanAnnouncer::new(listener.local_addr()?).await.ok();
    let mut announcement_timer = tokio::time::interval(Duration::from_secs(1));
    announcement_timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

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
                        let (returned, deliveries) = tokio::select! {
                            _ = &mut shutdown => break,
                            result = handle_command(session, connection, message, &writers) => result?,
                        };
                        session = returned;
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
                let kicked: HashSet<_> = deliveries.iter().filter(|delivery| matches!(delivery.message.event, ServerEvent::KickedFromRoom)).map(|delivery| delivery.recipient).collect();
                let replaced = authenticated
                    .iter()
                    .copied()
                    .filter(|candidate| {
                        *candidate != connection && !kicked.contains(candidate) && !session.is_current_connection(*candidate)
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
                for connection in kicked {
                    finish_connection(connection, &mut writers, &mut last_seen, &mut tasks, &mut authenticated);
                }
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
            _ = announcement_timer.tick() => {
                if let Some(announcer) = &announcer && let Some(info) = session.room_info() {
                    let _ = tokio::time::timeout(Duration::from_millis(250), announcer.announce(session.room_id(), info)).await;
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
