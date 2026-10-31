//! 四人双升与六人拖拉机的牌桌、亮主、埋底、出牌与闲家计分界面。
mod client;
pub(crate) use client::{ShengjiClientState, ShengjiScoreCaptureEffect};

pub(crate) mod actions;
mod assets;
mod audio;
mod card_face;
mod counter;
mod hand_interaction;
mod hints;
mod lobby;
mod missing_suits;
mod plugin;
mod presentation;
mod screen;
mod settlement;
mod skin;
mod snapshot;
mod state;
mod throw_feedback;
mod view;

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

pub(super) use screen::ShengjiScreen;
