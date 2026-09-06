//! 双升（升级）四人牌桌、亮主、埋底、出牌与闲家计分界面。

use bevy::audio::Volume;
use bevy::prelude::*;
use bevy::ui::{FocusPolicy, RelativeCursorPosition};
use leocard_client::{NetworkState, ShengjiScoreCaptureEffect};
use leocard_protocol::{
    ClientCommand, GameCommand, GameKind, MatchId, PlayerId, SeatId, ShengjiCommand,
    ShengjiFiveTrumpCrossingStage, ShengjiPhaseView, ShengjiPlayerState, ShengjiPublicPlay,
    ShengjiSnapshot, ShengjiThrowFailureStage,
};
use leocard_qigui523::{QiGuiRank, QiGuiSuit};
use leocard_shengji::{
    ShengjiBidTrump, ShengjiCard, ShengjiGreedyBot, ShengjiGreedyBotRequest, ShengjiRank,
    ShengjiRuleSet, ShengjiSuit, ShengjiThrowPenalty, ShengjiTrump,
};
use std::collections::HashSet;

use super::*;

const SHENGJI_SETTLEMENT_MODAL_DELAY: f32 = 3.38;
const SHENGJI_SETTLEMENT_ROW_INTERVAL: f32 = 0.18;

pub mod actions;
mod hints;
mod lobby;
mod presentation;
mod settlement;
mod snapshot;
mod state;
mod throw_feedback;
mod view;

pub use hints::*;
pub use lobby::*;
pub use presentation::*;
pub use settlement::*;
pub use snapshot::*;
pub use state::*;
pub use throw_feedback::*;
pub use view::*;
