use super::super::MahjongUiState;
use bevy::prelude::*;
use leocard_mahjong::{MahjongClaimOption, MahjongTile};
use leocard_protocol::{MahjongPhaseView, MahjongSnapshot, MatchId, PlayerId};

/// 标识一次实际显示操作按钮的机会，保持与界面实体的生命周期独立。
#[derive(Component, Clone, Copy, Eq, PartialEq)]
pub(in super::super) struct MahjongActionPrompt {
    match_id: MatchId,
    sequence_index: u8,
    discard_count: usize,
    wall_len: u16,
    tile: Option<MahjongTile>,
    robbing_source: Option<PlayerId>,
}

impl MahjongActionPrompt {
    pub(in super::super) fn from_snapshot(
        game: &MahjongSnapshot,
        ui: &MahjongUiState,
    ) -> Option<Self> {
        if game
            .players
            .iter()
            .any(|player| player.id == game.you && player.auto_play)
        {
            return None;
        }
        let tile = if let Some(pending) = &game.pending_claim {
            if pending.your_response.is_some()
                || pending.your_options.is_empty()
                || (ui.no_claim && !pending.your_options.contains(&MahjongClaimOption::Win))
            {
                return None;
            }
            Some(pending.tile)
        } else {
            if !matches!(game.phase, MahjongPhaseView::Playing)
                || game.current_player != game.you
                || !(game.can_self_draw
                    || !game.concealed_kong_options.is_empty()
                    || (!ui.no_claim && !game.added_kong_options.is_empty()))
            {
                return None;
            }
            game.your_drawn_tile.filter(|tile| {
                game.can_self_draw
                    || game.concealed_kong_options.contains(&tile.kind())
                    || (!ui.no_claim && game.added_kong_options.contains(tile))
            })
        };
        Some(Self {
            match_id: game.match_id,
            sequence_index: game.sequence_index,
            discard_count: game.discards.len(),
            wall_len: game.wall_len,
            tile,
            robbing_source: game
                .pending_claim
                .as_ref()
                .filter(|pending| pending.robbing_kong)
                .map(|pending| pending.source),
        })
    }

    pub(in super::super) fn robbing_tile_for(self, player: PlayerId) -> Option<MahjongTile> {
        if self.robbing_source == Some(player) {
            self.tile
        } else {
            None
        }
    }

    pub(in super::super) fn response_tile(self) -> Option<MahjongTile> {
        self.tile
    }
}

#[derive(Resource, Default)]
pub(in super::super) struct MahjongPromptPlayback {
    pub previous: Option<MahjongActionPrompt>,
}
