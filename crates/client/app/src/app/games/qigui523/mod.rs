//! 七鬼五二三客户端表现层。
//!
//! 纯规则位于独立游戏 crate；这里仅维护七鬼五二三特有的牌桌、出牌演出与
//! 结算界面。
mod client;
pub(crate) use client::{QiGuiClientState, ScoreCaptureEffect};

pub(crate) mod actions;
mod assets;
mod card_face;
mod cards;
mod center;
mod clock;
mod counter;
mod hand_interaction;
mod hints;
mod lobby;
mod play_effects;
mod player_stats;
mod players;
mod plays;
mod plugin;
mod rule_labels;
mod scores;
mod screen;
mod skin;
mod state;
pub(super) mod summary;
mod table;
mod turn_timer;
mod voices;

pub(crate) use actions::*;
pub(crate) use assets::*;
pub(crate) use cards::*;
use center::*;
pub(crate) use clock::*;
use hand_interaction::*;
pub(crate) use hints::*;
pub(crate) use lobby::*;
use play_effects::*;
use player_stats::*;
use players::*;
use plays::*;
pub(crate) use plugin::*;
pub(crate) use rule_labels::*;
use scores::*;
pub(super) use screen::*;
use skin::*;
pub(crate) use state::*;
pub(crate) use summary::*;
pub(crate) use table::*;
pub(crate) use turn_timer::*;
