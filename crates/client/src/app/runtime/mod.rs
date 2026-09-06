//! 应用启动、平台集成、持久化、资源与网络会话。

use bevy::prelude::*;

use super::*;

mod application;
mod platform;
mod preferences;
mod resources;
mod session;

pub use application::*;
pub use platform::*;
pub use preferences::*;
pub use resources::*;
pub use session::*;
