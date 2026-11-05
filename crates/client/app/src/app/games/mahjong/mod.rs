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
mod layout;
mod lobby;
mod modal;
mod players;
mod plugin;
mod presentation;
mod projection;
mod screen;
pub(super) mod settlement;
mod state;
mod status;
mod surfaces;
mod tile_voices;
mod tiles;
mod view;
mod voices;
mod wind_labels;
pub(crate) use crate::app::presentation::{
    MahjongTileHighlight, MahjongTileMaterial, MahjongTileRestRotation, MahjongTileSurface,
    attach_mahjong_tile_surface, mahjong_local_light, sync_mahjong_tile_orientation,
};
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
use modal::advance_mahjong_modal;
use players::*;
pub(crate) use plugin::*;
pub(crate) use presentation::*;
use projection::*;
pub(super) use screen::MahjongScreen;
use settlement::*;
pub(crate) use state::*;
use status::*;
use surfaces::*;
use tile_voices::*;
use tiles::*;
pub(crate) use view::*;
use voices::*;
use wind_labels::*;
