//! 跨游戏结算演出的运行时状态与 UI 标记组件。

use bevy::prelude::*;
use leocard_game_common::PlayerId;
use leocard_protocol::{MatchId, PlayerScore};

pub(crate) const SUMMARY_MODAL_ENTRY_DURATION: f32 = 0.55;
pub(crate) const SUMMARY_ROW_START_DELAY: f32 = 0.38;
pub(crate) const SUMMARY_ROW_INTERVAL: f32 = 0.18;
pub(crate) const SUMMARY_ROW_ENTRY_DURATION: f32 = 0.32;
pub(super) const SUMMARY_SCORE_COUNT_DURATION: f32 = 0.72;
pub(crate) const SUMMARY_ACTIONS_EXTRA_DELAY: f32 = 0.30;

#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct SummaryAnimationSet;

#[derive(Clone, Copy, Debug)]
pub(crate) struct SummaryDescriptor {
    pub match_id: MatchId,
    /// 同一场游戏内的结算序号，由各游戏自行提供。
    pub settlement_index: Option<u32>,
    pub entry_count: usize,
    pub nonnegative_outcome: bool,
    pub reveal_duration: f32,
}

/// Game adapters supply timing decisions; the renderer only consumes this projection.
#[derive(Resource, Default)]
pub(crate) struct SummaryPlayback {
    pub descriptor: Option<SummaryDescriptor>,
    pub pause_at: Option<f32>,
}

#[derive(Resource, Default)]
pub(crate) struct GameSummaryAnimation {
    pub match_id: Option<MatchId>,
    pub settlement_index: Option<u32>,
    pub entry_count: usize,
    pub elapsed: f32,
    pub nonnegative_outcome: bool,
    pub outcome_sound_played: bool,
}

#[derive(Component)]
pub(crate) struct AnimatedSummaryScore {
    pub target: i64,
    pub signed: bool,
    pub delay: f32,
    pub unit: &'static str,
}

#[derive(Clone, Copy)]
pub(crate) struct SummaryScore {
    pub player: PlayerId,
    pub score: i64,
    pub signed: bool,
}

impl From<PlayerScore> for SummaryScore {
    fn from(value: PlayerScore) -> Self {
        Self {
            player: value.player,
            score: i64::from(value.score),
            signed: false,
        }
    }
}

impl SummaryScore {
    pub(crate) fn signed(player: PlayerId, score: i32) -> Self {
        Self {
            player,
            score: i64::from(score),
            signed: true,
        }
    }
}

#[derive(Component)]
pub(crate) struct GameSummaryModal;

#[derive(Component)]
pub(crate) struct GameSummaryRow {
    pub delay: f32,
}

#[derive(Component)]
pub(crate) struct GameSummaryDivider {
    pub delay: f32,
}

#[derive(Component)]
pub(crate) struct GameSummaryActions {
    pub delay: f32,
}

#[derive(Component)]
pub(crate) struct AnimatedSummaryText {
    pub color: Color,
    pub delay: f32,
}

#[derive(Component)]
pub(crate) struct GameSummaryPanelTexture;
