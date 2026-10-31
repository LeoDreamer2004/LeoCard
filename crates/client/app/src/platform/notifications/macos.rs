use super::DesktopNotification;
use super::service::NotificationApplication;
use mac_usernotifications::{Notification, blocking, check_bundle};

pub(super) struct NotificationBackend;

impl NotificationBackend {
    pub(super) fn new(application: NotificationApplication) -> Result<Self, String> {
        // 裸 cargo run 没有应用包身份，必须先检查，避免原生框架因缺失标识而崩溃。
        check_bundle().map_err(|error| error.to_string())?;
        let _ = (application.id, application.name, application.icon);
        if !blocking::request_auth().map_err(|error| error.to_string())? {
            return Err("系统通知未获授权".to_owned());
        }
        Ok(Self)
    }

    pub(super) fn show(&mut self, notification: &DesktopNotification) -> Result<(), String> {
        // macOS 左侧图标来自 LeoCard.app 的 CFBundleIconFile。
        Notification::new()
            .title(&notification.title)
            .message(&notification.body)
            .send_blocking()
            .map(|_| ())
            .map_err(|error| error.to_string())
    }
}
