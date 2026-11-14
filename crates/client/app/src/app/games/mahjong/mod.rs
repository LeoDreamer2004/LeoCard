//! 麻将客户端表现层。

mod action_prompt;
pub(crate) mod actions;
mod assets;
mod audio;
mod auto;
mod catalog;
mod controls;
mod dice;
mod discard;
mod fan_guide;
mod hand;
mod hover;
mod layout;
mod lobby;
mod modal;
mod players;
mod plugin;
mod presentation;
mod ready_hand;
mod screen;
pub(super) mod settlement;
mod state;
mod status;
mod tile_voices;
mod tiles;
mod view;
mod voices;
mod wall;
mod wind_labels;

use crate::app::presentation::mahjong::{MahjongProjectionFrame, MahjongTableRoot};
use crate::app::presentation::{
    MahjongTileHighlight, MahjongTileMaterial, MahjongTileRestRotation, MahjongTileSurface,
    attach_mahjong_tile_surface, mahjong_local_light,
};
use action_prompt::*;
pub(crate) use actions::*;
pub(crate) use assets::*;
use audio::*;
use auto::*;
use controls::*;
use discard::*;
use fan_guide::*;
use hand::*;
pub(crate) use hover::MahjongFanCalculator;
use hover::*;
pub(crate) use lobby::*;
use modal::advance_mahjong_modal;
use players::*;
pub(crate) use plugin::*;
pub(crate) use presentation::*;
use ready_hand::*;
pub(super) use screen::MahjongScreen;
use settlement::*;
pub(crate) use state::*;
use status::*;
use tile_voices::*;
use tiles::*;
pub(crate) use view::*;
use voices::*;
use wind_labels::*;
