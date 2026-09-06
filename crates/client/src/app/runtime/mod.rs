//! 应用启动、平台集成、持久化、资源与网络会话。

mod application;
mod platform;
mod preferences;
mod resources;
mod session;

use super::*;
pub use application::*;
use bevy::prelude::*;
pub use platform::*;
pub use preferences::*;
pub use resources::*;
pub use session::*;
