use super::commands::handle_command;
use super::connection::{ConnectionTask, finish_connection, retire_connection};
use super::socket::serve_socket;
use super::tls::ServerTls;
use crate::TcpError;
use crate::discovery::LanAnnouncer;
use leocard_host::{ConnectionId, Delivery, HostSession};
use leocard_protocol::{ClientMessage, ServerEvent, ServerMessage};
use std::collections::{HashMap, HashSet};
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinSet;
use tokio::time::Instant;

pub(super) const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(3);

const CONNECTION_QUEUE: usize = 64;
const INBOUND_QUEUE: usize = 256;
const MAX_CONNECTIONS: usize = 16;
const CONNECTION_IDLE_TIMEOUT: Duration = Duration::from_secs(15);
const TURN_TIMER_INTERVAL: Duration = Duration::from_millis(100);

pub(super) enum Inbound {
    Message(ConnectionId, ClientMessage),
    Disconnected(ConnectionId),
}

pub(super) async fn run_server(
    listener: TcpListener,
    mut session: HostSession,
    mut shutdown: oneshot::Receiver<()>,
    tls: Option<ServerTls>,
) -> Result<(), TcpError> {
    let (inbound_tx, mut inbound_rx) = mpsc::channel(INBOUND_QUEUE);
    let mut writers: HashMap<ConnectionId, mpsc::Sender<ServerMessage>> = HashMap::new();
    let mut tasks: HashMap<ConnectionId, ConnectionTask> = HashMap::new();
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
                let (writer_tx, writer_rx) = mpsc::channel(CONNECTION_QUEUE);
                writers.insert(connection, writer_tx);
                last_seen.insert(connection, tokio::time::Instant::now());
                let task = tokio::spawn(serve_socket(
                    connection, stream, writer_rx, inbound_tx.clone(), tls.clone(),
                ));
                tasks.insert(connection, ConnectionTask(task));
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
    writers.clear();
    if flush_writers {
        let mut closing = JoinSet::new();
        for task in tasks.into_values() {
            closing.spawn(task.finish());
        }
        while closing.join_next().await.is_some() {}
    } else {
        for task in tasks.into_values() {
            task.abort();
        }
    }
    Ok(())
}

fn dispatch_deliveries(
    session: &mut HostSession,
    writers: &mut HashMap<ConnectionId, mpsc::Sender<ServerMessage>>,
    last_seen: &mut HashMap<ConnectionId, Instant>,
    tasks: &mut HashMap<ConnectionId, ConnectionTask>,
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
