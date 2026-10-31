#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
mod service;
#[cfg(target_os = "windows")]
mod windows;

pub use service::{DesktopNotification, DesktopNotifications, NotificationTicket};
