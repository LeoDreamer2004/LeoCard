//! 跨游戏结算演出的运行时状态与 UI 标记组件。

use super::*;
use leocard_protocol::MatchId;

#[derive(Resource, Default)]
pub struct GameSummaryAnimation {
    pub match_id: Option<MatchId>,
    pub texas_hand_number: Option<u32>,
    pub settlement_index: Option<u32>,
    pub entry_count: usize,
    pub elapsed: f32,
    pub nonnegative_outcome: bool,
    pub outcome_sound_played: bool,
}

#[derive(Component)]
pub struct AnimatedSummaryScore {
    pub target: u32,
    pub delay: f32,
}

#[derive(Component)]
pub struct AnimatedSignedSummaryScore {
    pub target: i32,
    pub delay: f32,
}

#[derive(Component)]
pub struct GameSummaryModal;

#[derive(Component)]
pub struct GameSummaryRow {
    pub delay: f32,
}

#[derive(Component)]
pub struct GameSummaryDivider {
    pub delay: f32,
}

#[derive(Component)]
pub struct GameSummaryActions {
    pub delay: f32,
}

#[derive(Component)]
pub struct AnimatedSummaryText {
    pub color: Color,
    pub delay: f32,
}

#[derive(Component)]
pub struct GameSummaryPanelTexture;
