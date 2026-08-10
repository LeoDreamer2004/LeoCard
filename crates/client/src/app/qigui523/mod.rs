//! 七鬼五二三客户端表现层。
//!
//! 纯规则位于独立游戏 crate；这里仅维护七鬼五二三特有的牌桌、出牌演出与
//! 结算界面。

use super::*;

mod effects;
mod play_effects;
mod summary;
mod table;

pub(in crate::app) use effects::*;
pub(in crate::app) use play_effects::*;
pub(in crate::app) use summary::*;
pub(in crate::app) use table::*;
