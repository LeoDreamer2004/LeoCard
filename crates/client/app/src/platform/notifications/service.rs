#[cfg(target_os = "linux")]
use super::linux::NotificationBackend;
#[cfg(target_os = "macos")]
use super::macos::NotificationBackend;
#[cfg(target_os = "windows")]
use super::windows::NotificationBackend;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

pub struct DesktopNotification {
    pub title: String,
    pub body: String,
}

/// 只取消尚未提交给操作系统的通知，不干涉系统通知中心里的历史记录。
pub struct NotificationTicket(Arc<AtomicBool>);

impl NotificationTicket {
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Relaxed);
    }
}

pub(super) struct NotificationApplication {
    pub id: String,
    pub name: String,
    pub icon: PathBuf,
}

struct NotificationRequest {
    notification: DesktopNotification,
    cancelled: Arc<AtomicBool>,
    minimum_interval: Duration,
}

/// 单个后台线程负责系统调用与授权，调用方不等待 D-Bus、WinRT 或授权对话框。
pub struct DesktopNotifications {
    requests: SyncSender<NotificationRequest>,
    errors: Mutex<Receiver<String>>,
}

impl DesktopNotifications {
    pub fn new(
        app_id: &str,
        app_name: &str,
        icon_png: &'static [u8],
        icon_path: PathBuf,
    ) -> io::Result<Self> {
        let (requests, receiver) = sync_channel::<NotificationRequest>(1);
        let (errors, error_receiver) = sync_channel(1);
        let application = NotificationApplication {
            id: app_id.to_owned(),
            name: app_name.to_owned(),
            icon: icon_path,
        };
        thread::Builder::new()
            .name("desktop-notifications".to_owned())
            .spawn(move || {
                let initialized = (|| {
                    let parent = application.icon.parent().ok_or("通知图标路径没有目录")?;
                    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
                    if fs::read(&application.icon).ok().as_deref() != Some(icon_png) {
                        fs::write(&application.icon, icon_png)
                            .map_err(|error| error.to_string())?;
                    }
                    // 保持单文件发布：图标来自程序内嵌 PNG，只向系统 API 提供缓存路径。
                    let path = application
                        .icon
                        .canonicalize()
                        .map_err(|error| error.to_string())?;
                    let application = NotificationApplication {
                        icon: path,
                        ..application
                    };
                    NotificationBackend::new(application)
                })();
                let mut backend = match initialized {
                    Ok(backend) => backend,
                    Err(error) => {
                        let _ = errors.try_send(error);
                        return;
                    }
                };
                let mut last_delivery: Option<Instant> = None;
                for request in receiver {
                    if request.cancelled.load(Ordering::Relaxed) {
                        continue;
                    }
                    if let Some(last) = last_delivery {
                        // 授权或系统调用可能延迟首次发送，冷却以实际提交时间再次兜底。
                        thread::sleep(request.minimum_interval.saturating_sub(last.elapsed()));
                    }
                    if !request.cancelled.load(Ordering::Relaxed) {
                        match backend.show(&request.notification) {
                            Ok(()) => last_delivery = Some(Instant::now()),
                            Err(error) => {
                                let _ = errors.try_send(error);
                            }
                        }
                    }
                }
            })?;
        Ok(Self {
            requests,
            errors: Mutex::new(error_receiver),
        })
    }

    pub fn show(
        &self,
        notification: DesktopNotification,
        minimum_interval: Duration,
    ) -> Result<NotificationTicket, String> {
        let cancelled = Arc::new(AtomicBool::new(false));
        self.requests
            .try_send(NotificationRequest {
                notification,
                cancelled: Arc::clone(&cancelled),
                minimum_interval,
            })
            .map_err(|error| format!("无法提交桌面通知：{error}"))?;
        Ok(NotificationTicket(cancelled))
    }

    pub fn take_error(&self) -> Option<String> {
        self.errors.lock().ok()?.try_recv().ok()
    }
}
