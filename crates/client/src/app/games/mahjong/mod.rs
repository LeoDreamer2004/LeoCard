//! 麻将客户端表现层。

use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;
use bevy::ui::FocusPolicy;
use leocard_mahjong::{
    MahjongClaim, MahjongClaimOption, MahjongDragon, MahjongFlower, MahjongKongKind,
    MahjongMatchLength, MahjongMeldKind, MahjongRuleSet, MahjongSuit, MahjongTile, MahjongTileKind,
    MahjongWind,
};
use leocard_protocol::{
    ClientCommand, GameCommand, MahjongCommand, MahjongEvent, MahjongPhaseView, MahjongSnapshot,
    MatchId, PlayerId,
};
use std::collections::VecDeque;

use super::*;

pub mod actions;
mod controls;
mod hand;
mod lobby;
mod material;
mod players;
mod presentation;
mod settlement;
mod state;
mod status;
mod tiles;
mod view;

use controls::*;
use hand::*;
pub use lobby::*;
pub use material::*;
use players::*;
pub use presentation::*;
use settlement::*;
pub use state::*;
use status::*;
pub use tiles::*;
pub use view::*;
