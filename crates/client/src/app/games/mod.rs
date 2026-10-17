//! 各棋牌游戏独立的客户端状态、输入和表现层。

pub(super) mod blackjack;
mod catalog;
mod composition;
pub(super) mod mahjong;
pub(super) mod minecraft_guess;
mod plugin;
pub(super) mod qigui523;
pub(super) mod shengji;
mod summary_sync;
pub(super) mod texas_holdem;
pub(super) mod uno;

pub(super) use catalog::*;
pub(super) use composition::*;
pub(super) use plugin::*;
pub(super) use summary_sync::sync_summary_playback;

mod screen_change;
pub(crate) use screen_change::GameScreenChange;
