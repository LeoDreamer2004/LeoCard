//! 麻将客户端快照观察状态。

use super::{MahjongOwnDiscardAnimation, MahjongRemoteDiscardAnimation};
use crate::app::presentation::{Observed, TableBackgroundMaterial};
use bevy::picking::hover::PickingInteraction;
use bevy::prelude::*;
use leocard_mahjong::{MahjongClaim, MahjongTile};
use leocard_protocol::{MatchId, PlayerId};
use std::collections::HashMap;

#[derive(Resource, Default)]
pub(crate) struct MahjongUiState {
    pub observed_table: Observed<(MatchId, u8), MahjongTableObservation>,
    pub intro_deal_match: Option<MatchId>,
    pub hand_hover_lifts: HashMap<i32, (f32, PickingInteraction)>,
    pub ready_hint_hovered: bool,
    pub fan_guide_open: bool,
    pub fan_guide_progress: f32,
    pub fan_guide_tier: u16,
    pub auto_drawer_open: bool,
    pub auto_win: bool,
    pub no_claim: bool,
    pub auto_draw_discard: bool,
    pub choice_menu: Option<MahjongChoiceMenu>,
    pub fan_summary_continued: Option<(MatchId, u8)>,
    pub final_summary_opened_at: Option<(MatchId, u8, f32)>,
    pub last_automatic_action: Option<MahjongAutomaticActionKey>,
    pub(super) auto_hand: Option<(MatchId, u8)>,
    pub table_material: Option<Handle<TableBackgroundMaterial>>,
    pub(super) last_table_height: Option<f32>,
    pub discard_animation: Option<MahjongOwnDiscardAnimation>,
    pub remote_discard_animation: Option<MahjongRemoteDiscardAnimation>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MahjongChoiceMenu {
    Chow {
        match_id: MatchId,
        source: PlayerId,
        tile: MahjongTile,
    },
    Kong {
        match_id: MatchId,
        sequence_index: u8,
        hand_len: usize,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MahjongAutomaticActionKey {
    Claim(MatchId, u8, MahjongTile, MahjongClaim),
    SelfDraw(MatchId, u8, MahjongTile),
    Discard(MatchId, u8, MahjongTile),
}

#[derive(Default)]
pub(crate) struct MahjongTableObservation {
    pub hand: Vec<MahjongTile>,
    pub counts: [u8; 4],
    pub flowers: [u8; 4],
    pub wall_len: Option<u16>,
    pub discard_count: Option<usize>,
    pub drawn_tile: Option<MahjongTile>,
    pub meld_count: usize,
}

impl MahjongUiState {
    pub(crate) fn begin_hand(&mut self, match_id: MatchId, sequence_index: u8) {
        let hand = (match_id, sequence_index);
        if self.auto_hand == Some(hand) {
            return;
        }
        self.auto_hand = Some(hand);
        self.auto_drawer_open = false;
        self.auto_win = false;
        self.no_claim = false;
        self.auto_draw_discard = false;
        self.choice_menu = None;
        self.fan_summary_continued = None;
        self.final_summary_opened_at = None;
        self.last_automatic_action = None;
    }

    pub(crate) fn clear(&mut self) {
        *self = Self::default();
    }
}
