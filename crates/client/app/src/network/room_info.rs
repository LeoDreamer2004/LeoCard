use super::NETWORK_ROOM_ID;
use leocard_platform::time::{Duration, Instant};
use leocard_protocol::RoomInfo;
use leocard_tcp::query_room_info;
use std::io;
use std::sync::Mutex;
use std::sync::mpsc::{self, Receiver};
use std::thread;
use tokio::net::lookup_host;
use tokio::sync::oneshot;
use tokio::task::JoinSet;
use tokio::time::timeout;

use super::state::RoomInfoResult;

/// Metadata requests never join a room or occupy a player seat.
pub(crate) struct RoomInfoQueries {
    receiver: Mutex<Receiver<RoomInfoResult>>,
    stop: Option<oneshot::Sender<()>>,
}

impl RoomInfoQueries {
    pub fn start(addresses: Vec<String>) -> io::Result<Self> {
        let (sender, receiver) = mpsc::channel();
        let (stop, stopped) = oneshot::channel();
        thread::Builder::new().name("leocard-room-info".to_owned()).spawn(move || {
            let Ok(runtime) = tokio::runtime::Builder::new_current_thread().enable_all().build() else {
                for address in addresses { let _ = sender.send(RoomInfoResult { address, info: None }); }
                return;
            };
            runtime.block_on(async move {
                let mut queries = JoinSet::new();
                let mut addresses = addresses.into_iter();
                let mut stopped = stopped;
                loop {
                    while queries.len() < 4 {
                        let Some(address) = addresses.next() else { break };
                        queries.spawn(async move {
                            let info = timeout(Duration::from_secs(3), query(&address)).await.ok().flatten();
                            RoomInfoResult { address, info }
                        });
                    }
                    if queries.is_empty() { break; }
                    tokio::select! {
                        _ = &mut stopped => break,
                        result = queries.join_next() => {
                            if let Some(Ok(result)) = result && sender.send(result).is_err() { break; }
                        }
                    }
                }
            });
        })?;
        Ok(Self {
            receiver: Mutex::new(receiver),
            stop: Some(stop),
        })
    }

    pub fn poll(&self) -> Vec<RoomInfoResult> {
        self.receiver
            .lock()
            .expect("room query results")
            .try_iter()
            .collect()
    }
}

impl Drop for RoomInfoQueries {
    fn drop(&mut self) {
        if let Some(stop) = self.stop.take() {
            let _ = stop.send(());
        }
    }
}

async fn query(address: &str) -> Option<(RoomInfo, u32)> {
    for endpoint in lookup_host(address).await.ok()? {
        let start = Instant::now();
        if let Ok(info) = query_room_info(endpoint, NETWORK_ROOM_ID).await {
            return Some((
                info,
                start.elapsed().as_millis().min(u128::from(u32::MAX)) as u32,
            ));
        }
    }
    None
}
