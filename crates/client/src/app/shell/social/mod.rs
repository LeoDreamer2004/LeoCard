//! 玩家互动投掷物与通用得分收集演出。

use bevy::audio::Volume;
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use bevy::window::PrimaryWindow;
use leocard_client::ScoreCaptureEffect;
use leocard_protocol::{
    ClientCommand, GameCommand, PlayerId, PlayerInteractionKind, QiGui523Command, ShengjiCommand,
    TexasHoldemCommand, UnoCommand,
};
use std::collections::HashMap;

use super::*;

pub const INTERACTION_COOLDOWN_MASK_FRAMES: usize = 48;

pub mod actions;
mod animation;
mod core;
mod state;

pub use animation::*;
pub use core::*;
pub use state::*;
