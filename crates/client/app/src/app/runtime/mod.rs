//! 应用启动、平台集成、持久化、资源与网络会话。

#[cfg(feature = "android")]
mod android;
mod application;
#[cfg(feature = "desktop")]
mod cursor;
#[cfg(feature = "android")]
mod native_text;
mod notifications;
mod platform;
mod plugin;
mod preferences;
mod projection;
mod resources;
mod session;
#[cfg(feature = "web")]
mod web;

pub(crate) use application::*;
use platform::*;
pub(super) use plugin::*;
pub(crate) use preferences::*;
pub(crate) use projection::*;
pub(crate) use resources::*;
pub(crate) use session::*;
