//! 麻将客户端表现层。

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

use super::*;
use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;
use bevy::ui::FocusPolicy;
use controls::*;
use hand::*;
pub use lobby::*;
pub use material::*;
use players::*;
pub use presentation::*;
use settlement::*;
pub use state::*;
use status::*;
use std::collections::VecDeque;
pub use tiles::*;
pub use view::*;
