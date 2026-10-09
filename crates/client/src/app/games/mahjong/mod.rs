//! 麻将客户端表现层。

mod action_prompt;
pub(crate) mod actions;
mod assets;
mod auto;
mod controls;
mod discard;
mod fan_guide;
mod hand;
mod hover;
mod lobby;
mod material;
mod modal;
mod players;
mod plugin;
mod presentation;
pub(super) mod settlement;
mod state;
mod status;
mod tiles;
mod view;
pub(super) mod violation;
mod voices;

use action_prompt::*;
pub(crate) use actions::*;
pub(crate) use assets::*;
use auto::*;
use controls::*;
use discard::*;
use fan_guide::*;
use hand::*;
pub(crate) use hover::MahjongFanCalculator;
use hover::*;
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
use voices::*;

use modal::advance_mahjong_modal;

mod hover_retention;
pub(crate) use hover_retention::MahjongHoverRetention;
