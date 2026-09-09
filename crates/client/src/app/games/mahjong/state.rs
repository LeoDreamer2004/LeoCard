//! 麻将客户端快照观察状态。

use crate::app::presentation::Observed;
use bevy::prelude::*;
use leocard_mahjong::MahjongTile;
use leocard_protocol::MatchId;

#[derive(Resource, Default)]
pub(crate) struct MahjongUiState {
    pub observed_table: Observed<(MatchId, u8), MahjongTableObservation>,
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
