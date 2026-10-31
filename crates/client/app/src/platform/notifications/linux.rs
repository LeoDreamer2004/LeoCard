use super::DesktopNotification;
use super::service::NotificationApplication;
use notify_rust::{Hint, Notification, NotificationHandle};

pub(super) struct NotificationBackend {
    application: NotificationApplication,
    // GNOME 等桌面在发送方断开 D-Bus 后会收回通知，因此保留最后一条的连接。
    handle: Option<NotificationHandle>,
}

impl NotificationBackend {
    pub(super) fn new(application: NotificationApplication) -> Result<Self, String> {
        Ok(Self {
            application,
            handle: None,
        })
    }

    pub(super) fn show(&mut self, notification: &DesktopNotification) -> Result<(), String> {
        let icon = self
            .application
            .icon
            .to_str()
            .ok_or("通知图标路径不是 UTF-8")?;
        let handle = Notification::new()
            .appname(&self.application.name)
            .summary(&notification.title)
            .body(&notification.body)
            .icon(icon)
            .hint(Hint::DesktopEntry(self.application.id.clone()))
            .timeout(8_000)
            .show()
            .map_err(|error| error.to_string())?;
        self.handle = Some(handle);
        Ok(())
    }
}
