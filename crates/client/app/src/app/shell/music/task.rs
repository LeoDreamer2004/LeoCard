use std::sync::Mutex;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;

// Replacing/dropping a task detaches its receiver. Stale results cannot change
// the current account, selected playlist or audio source.
pub(super) struct Task<T> {
    receiver: Mutex<Receiver<Result<T, String>>>,
}

impl<T: Send + 'static> Task<T> {
    pub fn start(
        work: impl FnOnce() -> Result<T, String> + Send + 'static,
    ) -> Result<Self, String> {
        let (sender, receiver) = mpsc::channel();
        thread::Builder::new()
            .name("leocard-music".to_owned())
            .spawn(move || {
                let _ = sender.send(work());
            })
            .map_err(|_| "无法启动音乐后台任务")?;
        Ok(Self {
            receiver: Mutex::new(receiver),
        })
    }

    pub fn poll(&self) -> Option<Result<T, String>> {
        let Ok(receiver) = self.receiver.lock() else {
            return Some(Err("音乐后台任务不可用".to_owned()));
        };
        match receiver.try_recv() {
            Ok(result) => Some(result),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => Some(Err("音乐后台任务已中断，请重试".to_owned())),
        }
    }
}
