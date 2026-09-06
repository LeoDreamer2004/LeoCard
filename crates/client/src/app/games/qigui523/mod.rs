//! 七鬼五二三客户端表现层。
//!
//! 纯规则位于独立游戏 crate；这里仅维护七鬼五二三特有的牌桌、出牌演出与
//! 结算界面。

use bevy::audio::Volume;
use bevy::prelude::*;
use bevy::ui::{FocusPolicy, RelativeCursorPosition};
use leocard_client::NetworkState;

use leocard_protocol::{
    ClientCommand, GameCommand, GameKind, GamePhaseView, PlayerId, PlayerPublicState, PublicPlay,
    PublicPlayRecord, QiGui523Command, QiGui523Snapshot, SeatId, TABLE_SEAT_COUNT, TurnTimerView,
};
use leocard_qigui523::{
    ClassifiedPlay, QiGui523Bot, QiGui523BotRequest, QiGuiCard, QiGuiPlayKind, QiGuiRuleSet,
    SameCardPolicy, classify, has_legal_response,
};

use super::*;

struct SeatVisuals<'a> {
    game: &'a QiGui523Snapshot,
    client: &'a ClientResource,
    ui: &'a UiAssets,
    avatars: &'a AvatarImages,
    interaction_menu_open: Option<PlayerId>,
    play_effect: Option<&'a ActivePlayEffect>,
    last_play: Option<&'a (PlayerId, PublicPlay)>,
    score_capture: &'a ScoreCaptureEffectState,
    start_transition_active: bool,
}

#[derive(Clone, Copy)]
enum ScoreCardsPopupPlacement {
    Opponent(SeatSide),
    Own,
}

pub mod actions;
mod cards;
mod center;
mod clock;
mod effects;
mod hints;
mod lobby;
mod play_effects;
mod players;
mod plays;
mod scores;
mod state;
mod summary;
mod table;
mod turn_timer;

pub use cards::*;
use center::*;
pub use clock::*;
pub use effects::*;
pub use hints::*;
pub use lobby::*;
pub use play_effects::*;
use players::*;
use plays::*;
use scores::*;
pub use state::*;
pub use summary::*;
pub use table::*;
pub use turn_timer::*;
