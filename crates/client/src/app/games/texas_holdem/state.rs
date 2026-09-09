//! 德州扑克下注输入与牌桌组件状态。

use crate::app::presentation::Observed;
use bevy::prelude::*;
use leocard_protocol::{MatchId, PlayerId, TexasHoldemSnapshot};

#[derive(Resource, Default)]
pub(crate) struct TexasHoldemUiState {
    pub raise_to: u32,
    pub observed_table: Observed<(MatchId, u32), usize>,
}

impl TexasHoldemUiState {
    pub(crate) fn reconcile(&mut self, _game: &TexasHoldemSnapshot) {}

    pub(crate) fn clear(&mut self) {
        self.observed_table.clear();
    }
}

#[derive(Resource, Default)]
pub(crate) struct TexasRaiseHoldState {
    pub direction: i8,
    pub step: u32,
    pub minimum: u32,
    pub maximum: u32,
    pub elapsed: f32,
    pub next_repeat: f32,
}

#[derive(Component, Clone, Copy)]
pub(crate) struct TexasRaiseAdjustButton {
    pub direction: i8,
    pub step: u32,
    pub minimum: u32,
    pub maximum: u32,
}

#[derive(Component)]
pub(crate) struct TexasPotDivider {
    pub old_layout: bool,
    pub elapsed: f32,
}

#[derive(Component)]
pub(crate) struct TexasPotHover {
    pub eligible: Vec<PlayerId>,
}

#[derive(Component, Clone, Copy)]
pub(crate) struct TexasPlayerPanel {
    pub player: PlayerId,
    pub base_border: Color,
}
