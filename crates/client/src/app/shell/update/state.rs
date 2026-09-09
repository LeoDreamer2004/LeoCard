//! 自动更新任务的状态与后台事件通道。

use super::download_latest_release;
use bevy::prelude::Resource;
use semver::Version;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) enum UpdateState {
    #[default]
    Idle,
    Checking,
    Downloading {
        version: Version,
        downloaded: u64,
        total: Option<u64>,
    },
    UpToDate {
        version: Version,
    },
    Ready {
        version: Version,
        staged: PathBuf,
    },
    Failed(String),
}

#[derive(Resource, Default)]
pub(crate) struct UpdateManager {
    pub state: UpdateState,
    pub dialog_open: bool,
    pub(super) receiver: Option<Mutex<Receiver<UpdateEvent>>>,
}

impl UpdateManager {
    pub(crate) fn begin_or_show(&mut self) {
        self.dialog_open = true;
        if matches!(
            self.state,
            UpdateState::Checking | UpdateState::Downloading { .. } | UpdateState::Ready { .. }
        ) {
            return;
        }
        if cfg!(debug_assertions) {
            self.state = UpdateState::Failed(
                "开发构建不会覆盖自身；请使用 GitHub Release 版本测试自动更新。".to_owned(),
            );
            return;
        }
        if !cfg!(any(target_os = "linux", target_os = "windows")) {
            self.state = UpdateState::Failed("当前平台暂不支持自动更新".to_owned());
            return;
        }

        let (sender, receiver) = mpsc::channel();
        self.receiver = Some(Mutex::new(receiver));
        self.state = UpdateState::Checking;
        thread::spawn(move || {
            if let Err(error) = download_latest_release(&sender) {
                let _ = sender.send(UpdateEvent::Failed(error));
            }
        });
    }

    pub(super) fn take_events(&mut self) -> (Vec<UpdateEvent>, bool) {
        let Some(receiver) = self.receiver.as_mut() else {
            return (Vec::new(), false);
        };
        let receiver = receiver
            .get_mut()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut events = Vec::new();
        loop {
            match receiver.try_recv() {
                Ok(event) => events.push(event),
                Err(TryRecvError::Empty) => return (events, false),
                Err(TryRecvError::Disconnected) => return (events, true),
            }
        }
    }
}

#[derive(Debug)]
pub(super) enum UpdateEvent {
    Available {
        version: Version,
        total: Option<u64>,
    },
    Progress {
        downloaded: u64,
        total: Option<u64>,
    },
    UpToDate {
        version: Version,
    },
    Ready {
        version: Version,
        staged: PathBuf,
    },
    Failed(String),
}
