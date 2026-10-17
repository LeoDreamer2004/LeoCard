//! UNO 客户端表现层。

pub(crate) mod actions;
mod assets;
mod audio;
mod card_paths;
mod cards;
mod controls;
mod hand;
mod interaction;
mod jump_in_device;
mod layout;
mod lobby;
mod material;
mod modal;
mod players;
mod plugin;
mod presentation;
pub(super) mod settlement;
mod state;
mod view;

pub(crate) use actions::*;
pub(crate) use assets::*;
pub(crate) use audio::*;
pub(crate) use cards::*;
use controls::*;
pub(crate) use hand::*;
pub(crate) use interaction::*;
pub(crate) use jump_in_device::UnoJumpInDevice;
use layout::opponent_position;
pub(crate) use lobby::*;
pub(crate) use material::*;
use players::*;
pub(crate) use plugin::*;
pub(crate) use presentation::*;
use settlement::*;
pub(crate) use state::*;
pub(crate) use view::*;

use modal::advance_uno_modal;

pub(crate) use card_paths::uno_card_asset_path;
