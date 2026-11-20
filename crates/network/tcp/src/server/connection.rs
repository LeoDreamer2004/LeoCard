use leocard_host::ConnectionId;
use leocard_protocol::ServerMessage;
use std::collections::{HashMap, HashSet};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio::time::{Instant, timeout};

pub(super) struct ConnectionTask(pub JoinHandle<()>);

impl ConnectionTask {
    pub(super) fn abort(self) {
        self.0.abort();
    }

    pub(super) async fn finish(mut self) {
        if timeout(Duration::from_secs(2), &mut self.0).await.is_err() {
            self.0.abort();
        }
    }
}

pub(super) fn retire_connection(
    connection: ConnectionId,
    writers: &mut HashMap<ConnectionId, mpsc::Sender<ServerMessage>>,
    last_seen: &mut HashMap<ConnectionId, Instant>,
    tasks: &mut HashMap<ConnectionId, ConnectionTask>,
    authenticated: &mut HashSet<ConnectionId>,
) {
    writers.remove(&connection);
    last_seen.remove(&connection);
    authenticated.remove(&connection);
    if let Some(task) = tasks.remove(&connection) {
        task.abort();
    }
}

/// Drop the sender so the writer drains the final message before closing the
/// socket. A slow client must not stall the host's other players.
pub(super) fn finish_connection(
    connection: ConnectionId,
    writers: &mut HashMap<ConnectionId, mpsc::Sender<ServerMessage>>,
    last_seen: &mut HashMap<ConnectionId, Instant>,
    tasks: &mut HashMap<ConnectionId, ConnectionTask>,
    authenticated: &mut HashSet<ConnectionId>,
) {
    writers.remove(&connection);
    last_seen.remove(&connection);
    authenticated.remove(&connection);
    if let Some(task) = tasks.remove(&connection) {
        tokio::spawn(task.finish());
    }
}
