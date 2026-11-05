//! 德州扑克下注输入与牌桌组件状态。

use super::TexasSpectatorUiState;
use super::betting::raise_sizing;
use crate::app::presentation::Observed;
use bevy::prelude::*;
use leocard_protocol::{MatchId, Revision, TexasSnapshot};

#[derive(Resource, Default)]
pub(crate) struct TexasUiState {
    pub raise_to: u32,
    pub raise_input_valid: bool,
    pub raise_editor_open: bool,
    pub raise_pending: Option<(Revision, u64)>,
    pub spectator: TexasSpectatorUiState,
    pub observed_table: Observed<(MatchId, u32), usize>,
}

impl TexasUiState {
    pub(crate) fn reconcile(&mut self, game: &TexasSnapshot, revision: Revision, rejection: u64) {
        if let Some((sent_revision, sent_rejection)) = self.raise_pending
            && (revision != sent_revision || rejection != sent_rejection)
        {
            self.raise_pending = None;
            if revision != sent_revision {
                self.raise_editor_open = false;
            }
        }
        if raise_sizing(game).is_none() {
            self.raise_editor_open = false;
            self.raise_pending = None;
        }
    }

    pub(crate) fn clear(&mut self) {
        self.raise_editor_open = false;
        self.raise_pending = None;
        self.observed_table.clear();
        self.spectator.clear();
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
