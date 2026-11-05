use bevy::prelude::*;
use leocard_mahjong_link::LinkCell;
use leocard_protocol::{MahjongLinkSnapshot, MatchId};

#[derive(Resource, Default)]
pub(super) struct MahjongLinkUiState {
    pub selected: Option<LinkCell>,
    pub distance: u8,
    pub mode_menu_open: bool,
    pub guide_open: bool,
    observed: Option<(MatchId, u32, bool)>,
}

impl MahjongLinkUiState {
    pub fn reconcile(&mut self, game: &MahjongLinkSnapshot) {
        let current = (game.match_id, game.turn, game.push_used);
        if self.observed != Some(current) {
            self.selected = None;
            self.distance = 1;
            self.observed = Some(current);
        }
    }

    pub fn clear_game(&mut self) {
        self.observed = None;
        self.selected = None;
        self.distance = 1;
    }
}
