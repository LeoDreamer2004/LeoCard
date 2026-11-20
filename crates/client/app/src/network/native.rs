use super::state::{
    COMMAND_QUEUE, CommandSender, EventQueue, NetworkEvent, NetworkLaunch, NetworkStartError,
    push_event,
};
use super::worker::run_network;
use leocard_protocol::ClientMessage;
use std::thread;
use tokio::runtime;
use tokio::sync::mpsc;

pub(super) type WorkerHandle = thread::JoinHandle<()>;

pub(super) fn spawn_worker(
    launch: NetworkLaunch,
    events: EventQueue,
    reconnect_join: ClientMessage,
) -> Result<(CommandSender, WorkerHandle), NetworkStartError> {
    let (command_tx, command_rx) = mpsc::channel(COMMAND_QUEUE);
    let worker_events = events;
    let worker = thread::Builder::new()
        .name("leocard-network".to_owned())
        .spawn(move || {
            let runtime = match runtime::Builder::new_current_thread().enable_all().build() {
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

    Ok((command_tx, worker))
}

pub(super) fn remembered_reconnect_token(_launch: &NetworkLaunch) -> Option<u64> {
    None
}

pub(super) fn forget_session() {}
