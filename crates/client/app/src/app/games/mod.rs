//! 各棋牌游戏独立的客户端状态、输入和表现层。

pub(super) mod blackjack;
mod catalog;
mod composition;
pub(super) mod doudizhu;
pub(super) mod mahjong;
pub(super) mod mc_guess;
mod plugin;
pub(super) mod qigui523;
mod screen;
mod screen_change;
pub(super) mod shengji;
mod summary_sync;
pub(super) mod texas;
pub(super) mod uno;

pub(super) use catalog::*;
pub(super) use composition::*;
pub(super) use plugin::*;
pub(super) use screen::{GameLobbyContext, GameScreen, GameTableContext};
pub(crate) use screen_change::GameScreenChange;
pub(super) use summary_sync::sync_summary_playback;
