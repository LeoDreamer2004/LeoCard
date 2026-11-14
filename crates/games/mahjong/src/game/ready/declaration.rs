use super::super::{MahjongActionOutcome, MahjongGameError, MahjongGameState, validate_player};
use crate::{MahjongTile, MahjongTileKind, waiting_tile_kinds};
use leocard_game_common::PlayerId;
use std::collections::HashMap;

impl MahjongGameState {
    /// 报听只检查弃牌后的结构听牌，起和番数在真正和牌时判断。
    pub fn declared_ready_discards(&self, player: PlayerId) -> Vec<MahjongTile> {
        if self.ensure_playing_turn(player).is_err() || !self.rules.declared_ready {
            return Vec::new();
        }
        let state = &self.players[player.index()];
        if state.dead_hand || state.has_declared_ready() {
            return Vec::new();
        }
        let mut eligible = HashMap::new();
        state
            .hand
            .iter()
            .copied()
            .filter(|tile| {
                *eligible
                    .entry(tile.kind())
                    .or_insert_with(|| self.ready_waits_after_discard(player, *tile).is_some())
            })
            .collect()
    }

    pub fn declare_ready(
        &mut self,
        player: PlayerId,
        discard: MahjongTile,
    ) -> Result<MahjongActionOutcome, MahjongGameError> {
        self.ensure_playing_turn(player)?;
        if !self.declared_ready_discards(player).contains(&discard) {
            return Err(MahjongGameError::CannotDeclareReady);
        }
        let waits = self
            .ready_waits_after_discard(player, discard)
            .ok_or(MahjongGameError::CannotDeclareReady)?;
        let outcome = self.discard(player, discard)?;
        self.players[player.index()].ready_waits = Some(waits);
        Ok(outcome)
    }

    fn ready_waits_after_discard(
        &self,
        player: PlayerId,
        discard: MahjongTile,
    ) -> Option<Vec<MahjongTileKind>> {
        validate_player(player).ok()?;
        let state = &self.players[player.index()];
        let position = state.hand.iter().position(|tile| *tile == discard)?;
        let concealed = state
            .hand
            .iter()
            .enumerate()
            .filter_map(|(index, tile)| (index != position).then_some(tile.kind()))
            .collect::<Vec<_>>();
        let waits = waiting_tile_kinds(&concealed, &state.melds);
        (!waits.is_empty()).then_some(waits)
    }
}
