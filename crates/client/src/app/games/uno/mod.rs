//! UNO 客户端表现层。

pub mod actions;
mod audio;
mod cards;
mod controls;
mod hand;
mod interaction;
mod lobby;
mod material;
mod players;
mod presentation;
mod settlement;
mod state;
mod view;

use super::*;
pub use audio::*;
use bevy::audio::Volume;
use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;
use bevy::ui::FocusPolicy;
pub use cards::*;
use controls::*;
pub use hand::*;
pub use interaction::*;
pub use lobby::*;
pub use material::*;
use players::*;
pub use presentation::*;
use settlement::*;
pub use state::*;
use std::collections::{HashMap, HashSet, VecDeque};
pub use view::*;

/// UNO 最后一张牌的飞行动画结束后，完整公开牌桌两秒再进入结算。
pub const UNO_PLAY_CARD_DURATION: f32 = 0.58;
pub const UNO_FINISH_REVEAL_DURATION: f32 = UNO_PLAY_CARD_DURATION + 2.0;
