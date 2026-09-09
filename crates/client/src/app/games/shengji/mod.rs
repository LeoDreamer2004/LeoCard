//! 双升（升级）四人牌桌、亮主、埋底、出牌与闲家计分界面。

pub(crate) mod actions;
mod assets;
mod hand_interaction;
mod hints;
mod lobby;
mod plugin;
mod presentation;
mod settlement;
mod snapshot;
mod state;
mod throw_feedback;
mod view;
pub(super) mod violation;

pub(crate) use actions::*;
pub(crate) use assets::*;
pub(crate) use hand_interaction::*;
pub(crate) use hints::*;
pub(crate) use lobby::*;
pub(crate) use plugin::*;
pub(crate) use presentation::*;
pub(crate) use settlement::*;
pub(crate) use snapshot::*;
pub(crate) use state::*;
pub(crate) use throw_feedback::*;
pub(crate) use view::*;
