//! 麻将客户端快照观察状态。

use super::*;
use leocard_mahjong::MahjongTile;
use leocard_protocol::MatchId;

#[derive(Default)]
pub struct MahjongUiState {
    pub observed_match: Option<MatchId>,
    pub observed_sequence: u8,
    pub observed_hand: Vec<MahjongTile>,
    pub observed_counts: [u8; 4],
    pub observed_flowers: [u8; 4],
}
