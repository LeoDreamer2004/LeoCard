//! 跨游戏结算演出的运行时状态与 UI 标记组件。

use bevy::prelude::*;
use leocard_protocol::MatchId;

pub(crate) const SUMMARY_MODAL_ENTRY_DURATION: f32 = 0.55;
pub(crate) const SUMMARY_HAND_REVEAL_DURATION: f32 = 1.5;
pub(crate) const SUMMARY_ROW_START_DELAY: f32 = 0.38;
pub(crate) const SUMMARY_ROW_INTERVAL: f32 = 0.18;
pub(crate) const SUMMARY_ROW_ENTRY_DURATION: f32 = 0.32;
pub(super) const SUMMARY_SCORE_COUNT_DURATION: f32 = 0.72;
pub(crate) const SUMMARY_ACTIONS_EXTRA_DELAY: f32 = 0.30;
pub(crate) const TEXAS_UNCONTESTED_REVEAL_DURATION: f32 = 0.8;

#[derive(Clone, Copy, Debug)]
pub(crate) struct SummaryDescriptor {
    pub match_id: MatchId,
    pub texas_hand_number: Option<u32>,
    pub settlement_index: Option<u32>,
    pub entry_count: usize,
    pub nonnegative_outcome: bool,
    pub reveal_duration: f32,
}

#[derive(Resource, Default)]
pub(crate) struct GameSummaryAnimation {
    pub match_id: Option<MatchId>,
    pub texas_hand_number: Option<u32>,
    pub settlement_index: Option<u32>,
    pub entry_count: usize,
    pub elapsed: f32,
    pub nonnegative_outcome: bool,
    pub outcome_sound_played: bool,
}

#[derive(Component)]
pub(crate) struct AnimatedSummaryScore {
    pub target: u32,
    pub delay: f32,
}

#[derive(Component)]
pub(crate) struct AnimatedSignedSummaryScore {
    pub target: i32,
    pub delay: f32,
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
