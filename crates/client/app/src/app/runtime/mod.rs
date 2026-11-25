//! 应用启动、平台集成、持久化、资源与网络会话。

#[cfg(not(target_arch = "wasm32"))]
mod ai;
#[cfg(feature = "android")]
mod android;
mod application;
#[cfg(feature = "desktop")]
mod cursor;
mod display;
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

#[cfg(not(target_arch = "wasm32"))]
pub(crate) use ai::{AiModels, ModelStatus, initialize_ai_models, poll_ai_models, sync_lobby_ai};
pub(crate) use application::*;
pub(crate) use display::ToggleFullscreen;
use platform::*;
pub(super) use plugin::*;
pub(crate) use preferences::*;
pub(crate) use projection::*;
pub(crate) use resources::*;
pub(crate) use session::*;
