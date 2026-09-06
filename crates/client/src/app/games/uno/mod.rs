//! UNO 客户端表现层。

use bevy::audio::Volume;
use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;
use bevy::ui::FocusPolicy;
use leocard_client::NetworkState;
use leocard_protocol::{
    ClientCommand, GameCommand, GameKind, PlayerId, UnoCommand, UnoEvent, UnoPendingSwapView,
    UnoPhaseView, UnoPlayerState, UnoSnapshot,
};
use leocard_uno::{
    UnoCard, UnoChallengeResult, UnoColor, UnoDirection, UnoFace, UnoFlipSide, UnoPendingDrawKind,
    UnoRuleSet,
};
use std::collections::{HashMap, HashSet, VecDeque};

use super::*;

/// UNO 最后一张牌的飞行动画结束后，完整公开牌桌两秒再进入结算。
pub const UNO_PLAY_CARD_DURATION: f32 = 0.58;
pub const UNO_FINISH_REVEAL_DURATION: f32 = UNO_PLAY_CARD_DURATION + 2.0;

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

pub use audio::*;
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
pub use view::*;
