//! 各棋牌游戏独立的客户端状态、输入和表现层。

mod catalog;
mod composition;
pub(super) mod mahjong;
mod plugin;
pub(super) mod qigui523;
pub(super) mod shengji;
mod summary;
pub(super) mod texas_holdem;
pub(super) mod uno;
mod violations;

pub(super) use catalog::*;
pub(super) use composition::*;
pub(super) use plugin::*;
pub(super) use violations::*;

mod screen_change;
pub(crate) use screen_change::GameScreenChange;
