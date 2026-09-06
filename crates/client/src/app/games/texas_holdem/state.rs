//! 德州扑克下注输入与牌桌组件状态。

use super::*;
use leocard_protocol::{MatchId, PlayerId};

#[derive(Default)]
pub struct TexasHoldemUiState {
    pub raise_to: u32,
    pub observed_match: Option<MatchId>,
    pub observed_hand_number: u32,
    pub observed_community_len: usize,
}

#[derive(Resource, Default)]
pub struct TexasRaiseHoldState {
    pub direction: i8,
    pub step: u32,
    pub minimum: u32,
    pub maximum: u32,
    pub elapsed: f32,
    pub next_repeat: f32,
}

#[derive(Component, Clone, Copy)]
pub struct TexasRaiseAdjustButton {
    pub direction: i8,
    pub step: u32,
    pub minimum: u32,
    pub maximum: u32,
}

#[derive(Component)]
pub struct TexasPotDivider {
    pub old_layout: bool,
    pub elapsed: f32,
}

#[derive(Component)]
pub struct TexasPotHover {
    pub eligible: Vec<PlayerId>,
}

#[derive(Component, Clone, Copy)]
pub struct TexasPlayerPanel {
    pub player: PlayerId,
    pub base_border: Color,
}
