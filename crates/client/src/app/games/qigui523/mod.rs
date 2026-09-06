//! 七鬼五二三客户端表现层。
//!
//! 纯规则位于独立游戏 crate；这里仅维护七鬼五二三特有的牌桌、出牌演出与
//! 结算界面。

pub mod actions;
mod cards;
mod center;
mod clock;
mod effects;
mod hints;
mod lobby;
mod play_effects;
mod players;
mod plays;
mod scores;
mod state;
mod summary;
mod table;
mod turn_timer;

use super::*;
use bevy::audio::Volume;
use bevy::prelude::*;
use bevy::ui::{FocusPolicy, RelativeCursorPosition};
pub use cards::*;
use center::*;
pub use clock::*;
pub use effects::*;
pub use hints::*;
pub use lobby::*;
pub use play_effects::*;
use players::*;
use plays::*;
use scores::*;
pub use state::*;
pub use summary::*;
pub use table::*;
pub use turn_timer::*;
