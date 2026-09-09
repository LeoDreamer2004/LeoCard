//! 七鬼五二三客户端表现层。
//!
//! 纯规则位于独立游戏 crate；这里仅维护七鬼五二三特有的牌桌、出牌演出与
//! 结算界面。

pub(crate) mod actions;
mod assets;
mod cards;
mod center;
mod clock;
mod effects;
mod hand_interaction;
mod hints;
mod lobby;
mod play_effects;
mod players;
mod plays;
mod plugin;
mod scores;
mod state;
pub(super) mod summary;
mod table;
mod turn_timer;
pub(super) mod violation;

pub(crate) use actions::*;
pub(crate) use assets::*;
pub(crate) use cards::*;
use center::*;
pub(crate) use clock::*;
use effects::*;
use hand_interaction::*;
pub(crate) use hints::*;
pub(crate) use lobby::*;
pub(crate) use play_effects::*;
use players::*;
use plays::*;
pub(crate) use plugin::*;
use scores::*;
pub(crate) use state::*;
pub(crate) use summary::*;
pub(crate) use table::*;
pub(crate) use turn_timer::*;
