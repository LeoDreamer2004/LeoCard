//! 麻将客户端表现层。

pub(crate) mod actions;
mod assets;
mod controls;
mod hand;
mod lobby;
mod material;
mod players;
mod plugin;
mod presentation;
pub(super) mod settlement;
mod state;
mod status;
mod tiles;
mod view;
pub(super) mod violation;

pub(crate) use actions::*;
pub(crate) use assets::*;
use controls::*;
use hand::*;
pub(crate) use lobby::*;
pub(crate) use material::*;
use players::*;
pub(crate) use plugin::*;
pub(crate) use presentation::*;
use settlement::*;
pub(crate) use state::*;
use status::*;
use tiles::*;
pub(crate) use view::*;
