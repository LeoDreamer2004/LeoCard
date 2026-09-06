//! 跨游戏共用的错误与断线覆盖层。

use bevy::prelude::*;
use bevy::ui::FocusPolicy;

use super::*;

mod feedback;
mod rejection;
mod state;
mod view;

pub use feedback::*;
pub use rejection::*;
pub use state::*;
pub use view::*;
