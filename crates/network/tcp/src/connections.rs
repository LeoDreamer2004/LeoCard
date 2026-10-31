use crate::ConnectionTasks;
use leocard_host::ConnectionId;
use leocard_protocol::ServerMessage;
use std::collections::{HashMap, HashSet};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::{Instant, timeout};

pub(crate) fn retire_connection(
    connection: ConnectionId,
    writers: &mut HashMap<ConnectionId, mpsc::Sender<ServerMessage>>,
    last_seen: &mut HashMap<ConnectionId, Instant>,
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

/// Drop the sender so the writer drains the final message before closing the
/// socket. A slow client must not stall the host's other players.
pub(crate) fn finish_connection(
    connection: ConnectionId,
    writers: &mut HashMap<ConnectionId, mpsc::Sender<ServerMessage>>,
    last_seen: &mut HashMap<ConnectionId, Instant>,
    tasks: &mut HashMap<ConnectionId, ConnectionTasks>,
    authenticated: &mut HashSet<ConnectionId>,
) {
    writers.remove(&connection);
    last_seen.remove(&connection);
    authenticated.remove(&connection);
    if let Some((reader, mut writer)) = tasks.remove(&connection) {
        reader.abort();
        tokio::spawn(async move {
            if timeout(Duration::from_secs(2), &mut writer).await.is_err() {
                writer.abort();
            }
        });
    }
}
