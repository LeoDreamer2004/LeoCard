use super::DesktopNotification;
use super::service::NotificationApplication;
use tauri_winrt_notification::{Duration, IconCrop, Toast};
use winreg::RegKey;
use winreg::enums::HKEY_CURRENT_USER;

pub(super) struct NotificationBackend {
    application: NotificationApplication,
}

impl NotificationBackend {
    pub(super) fn new(mut application: NotificationApplication) -> Result<Self, String> {
        // WinRT 图片 URI 不接受 canonicalize 生成的扩展路径前缀。
        let icon = application
            .icon
            .to_string_lossy()
            .trim_start_matches(r"\\?\")
            .to_owned();
        application.icon = icon.into();
        // 便携 EXE 也使用自己的通知身份，不借用 PowerShell 等其他程序的名字。
        let (key, _) = RegKey::predef(HKEY_CURRENT_USER)
            .create_subkey(format!(
                r"Software\Classes\AppUserModelId\{}",
                application.id
            ))
            .map_err(|error| error.to_string())?;
        key.set_value("DisplayName", &application.name)
            .map_err(|error| error.to_string())?;
        key.set_value("IconUri", &application.icon.to_string_lossy().as_ref())
            .map_err(|error| error.to_string())?;
        Ok(Self { application })
    }

    pub(super) fn show(&mut self, notification: &DesktopNotification) -> Result<(), String> {
        Toast::new(&self.application.id)
            .title(&notification.title)
            .text1(&notification.body)
            .icon(
                &self.application.icon,
                IconCrop::Square,
                &self.application.name,
            )
            .duration(Duration::Short)
            .show()
            .map_err(|error| error.to_string())
    }
}
