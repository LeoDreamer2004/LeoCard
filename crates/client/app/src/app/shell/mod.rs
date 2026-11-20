//! 连接、大厅、设置以及跨游戏的产品功能。

mod achievements;
pub(super) mod actions;
pub(super) mod chat;
mod composition;
mod connection;
mod developer;
pub(super) mod input;
mod lobby;
mod navigation;
pub(super) mod overlays;
mod plugin;
pub(super) mod profile;
#[cfg(feature = "desktop")]
mod sakura_frp;
mod settings;
mod shop;
pub(super) mod social;
pub(super) mod state;
mod ui;
#[cfg(not(feature = "web"))]
pub(super) mod update;

pub(crate) use achievements::*;
pub(crate) use actions::*;
pub(crate) use chat::*;
pub(crate) use composition::*;
pub(crate) use connection::*;
pub(crate) use developer::*;
pub(crate) use input::*;
pub(crate) use lobby::*;
pub(crate) use navigation::*;
pub(crate) use overlays::*;
pub(super) use plugin::*;
pub(crate) use profile::*;
#[cfg(feature = "desktop")]
pub(crate) use sakura_frp::*;
pub(crate) use settings::*;
pub(crate) use shop::*;
pub(crate) use social::*;
pub(crate) use state::*;
pub(crate) use ui::*;
#[cfg(not(feature = "web"))]
pub(crate) use update::*;
