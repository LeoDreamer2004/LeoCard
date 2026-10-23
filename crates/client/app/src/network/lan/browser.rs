use super::worker::browse;
use leocard_protocol::RoomInfo;
use std::io;
use std::net::SocketAddr;
use std::thread;
use tokio::sync::{oneshot, watch};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LanRoom {
    pub server_id: u64,
    pub address: SocketAddr,
    pub info: RoomInfo,
}

#[derive(Clone, Default)]
pub(super) struct BrowserSnapshot {
    pub rooms: Vec<LanRoom>,
    pub error: Option<String>,
}

/// Runs only while a room picker is open. The UI sees snapshots, never socket state.
pub struct LanRoomBrowser {
    receiver: watch::Receiver<BrowserSnapshot>,
    snapshot: BrowserSnapshot,
    stop: Option<oneshot::Sender<()>>,
    closed: bool,
}

impl LanRoomBrowser {
    pub fn start() -> io::Result<Self> {
        let (sender, receiver) = watch::channel(BrowserSnapshot::default());
        let (stop, stopped) = oneshot::channel();
        thread::Builder::new()
            .name("leocard-lan-browser".to_owned())
            .spawn(move || {
                let result = match tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                {
                    Ok(runtime) => runtime.block_on(browse(&sender, stopped)),
                    Err(error) => Err(error),
                };
                if let Err(error) = result {
                    sender.send_replace(BrowserSnapshot {
                        rooms: Vec::new(),
                        error: Some(format!("无法搜索局域网房间：{error}")),
                    });
                }
            })?;
        Ok(Self {
            receiver,
            snapshot: BrowserSnapshot::default(),
            stop: Some(stop),
            closed: false,
        })
    }

    pub fn rooms(&self) -> &[LanRoom] {
        &self.snapshot.rooms
    }

    pub fn error(&self) -> Option<&str> {
        self.snapshot.error.as_deref()
    }

    pub fn poll(&mut self) -> bool {
        if self.closed {
            return false;
        }
        match self.receiver.has_changed() {
            Ok(false) => return false,
            Err(_) => self.closed = true,
            _ => {}
        }
        // A final error snapshot remains readable even after the worker exits.
        let mut snapshot = self.receiver.borrow_and_update().clone();
        if self.closed && snapshot.error.is_none() {
            snapshot.error = Some("局域网搜索已停止，请刷新重试".to_owned());
        }
        let changed =
            snapshot.rooms != self.snapshot.rooms || snapshot.error != self.snapshot.error;
        self.snapshot = snapshot;
        changed
    }
}

impl Drop for LanRoomBrowser {
    fn drop(&mut self) {
        if let Some(stop) = self.stop.take() {
            let _ = stop.send(());
        }
    }
}
