//! 玩家档案界面与各游戏长期统计。

mod rating;
mod state;
mod stats;
mod view;

use super::*;
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
pub use rating::*;
pub use state::*;
pub use stats::*;
pub use view::*;
