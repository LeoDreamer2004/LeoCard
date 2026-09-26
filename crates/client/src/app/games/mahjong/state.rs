//! 麻将客户端快照观察状态。

use crate::app::presentation::{Observed, TableBackgroundMaterial};
use bevy::prelude::*;
use leocard_mahjong::{MahjongClaim, MahjongTile};
use leocard_protocol::MatchId;

#[derive(Resource, Default)]
pub(crate) struct MahjongUiState {
    pub observed_table: Observed<(MatchId, u8), MahjongTableObservation>,
    pub fan_guide_open: bool,
    pub fan_guide_tier: u16,
    pub auto_drawer_open: bool,
    pub auto_win: bool,
    pub no_claim: bool,
    pub auto_draw_discard: bool,
    pub last_automatic_action: Option<MahjongAutomaticActionKey>,
    pub table_material: Option<Handle<TableBackgroundMaterial>>,
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
}

impl MahjongUiState {
    pub(crate) fn clear(&mut self) {
        *self = Self::default();
    }
}
