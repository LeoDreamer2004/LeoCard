use crate::{HEARTBEAT_INTERVAL, TcpError};
use leocard_host::{ConnectionId, Delivery, HostSession};
use leocard_protocol::{ClientMessage, ServerMessage};
use std::collections::HashMap;
use tokio::sync::mpsc;
use tokio::task;
use tokio::time::{MissedTickBehavior, interval};

/// Commands remain serial, while CPU work cannot starve socket I/O and keepalives.
pub(crate) async fn handle_command(
    mut session: HostSession,
    connection: ConnectionId,
    message: ClientMessage,
    writers: &HashMap<ConnectionId, mpsc::Sender<ServerMessage>>,
) -> Result<(HostSession, Vec<Delivery>), TcpError> {
    // These messages describe the last published revision, before the command completes.
    let heartbeats = session.heartbeat();
    let mut work = task::spawn_blocking(move || {
        let deliveries = session.handle(connection, message);
        (session, deliveries)
    });
    let mut timer = interval(HEARTBEAT_INTERVAL);
    timer.set_missed_tick_behavior(MissedTickBehavior::Skip);
    timer.tick().await;
    loop {
        tokio::select! {
            result = &mut work => {
                return result.map_err(|error| TcpError::ServerTask(error.to_string()));
            }
            _ = timer.tick() => {
                for delivery in &heartbeats {
                    if let Some(writer) = writers.get(&delivery.recipient) {
                        // Full/closed writers are retired by the normal delivery path.
                        let _ = writer.try_send(delivery.message.clone());
                    }
                }
            }
        }
    }
}
