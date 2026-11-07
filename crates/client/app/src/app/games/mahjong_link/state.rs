use super::input::{LinkPushGesture, LinkPushPreview};
use bevy::prelude::*;
use leocard_mahjong_link::LinkCell;
use leocard_protocol::{MahjongLinkSnapshot, MatchId};

#[derive(Resource, Default)]
pub(super) struct MahjongLinkUiState {
    pub selected: Option<LinkCell>,
    pub drag: Option<LinkPushGesture>,
    pub pending_push: Option<(u64, LinkPushPreview)>,
    observed: Option<(MatchId, u32, bool)>,
}

impl MahjongLinkUiState {
    pub fn reconcile(&mut self, game: &MahjongLinkSnapshot) {
        let current = (game.match_id, game.turn, game.push_used);
        if self.observed != Some(current) || !can_act(game) {
            self.clear_interaction();
            self.observed = Some(current);
        }
    }

    pub fn clear_game(&mut self) {
        self.observed = None;
        self.clear_interaction();
    }

    fn clear_interaction(&mut self) {
        self.selected = None;
        self.drag = None;
        self.pending_push = None;
    }

    pub fn preview(&self) -> Option<&LinkPushPreview> {
        self.pending_push
            .as_ref()
            .map(|(_, preview)| preview)
            .or_else(|| self.drag.as_ref()?.preview.as_ref())
    }
}

pub(super) fn can_act(game: &MahjongLinkSnapshot) -> bool {
    !game.match_complete
        && game.current_player == Some(game.you)
        && game
            .players
            .iter()
            .any(|player| player.player.id == game.you && player.connected && !player.auto_play)
}
