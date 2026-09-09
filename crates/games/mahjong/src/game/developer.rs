use super::{GameError, GameState, MahjongHandReplacementError, Phase, validate_player};
use crate::{MahjongPlayerId, MahjongTileKind};
use std::collections::HashSet;

impl GameState {
    pub fn replace_player_hand_from_wall(
        &mut self,
        player: MahjongPlayerId,
        kinds: &[MahjongTileKind],
    ) -> Result<(), GameError> {
        validate_player(player)?;
        if !matches!(self.phase, Phase::Playing) {
            return Err(GameError::WrongPhase);
        }
        let old_hand = &self.players[player.0].hand;
        if kinds.len() != old_hand.len() {
            return Err(GameError::InvalidHandReplacement(
                MahjongHandReplacementError::WrongTileCount {
                    expected: old_hand.len() as u16,
                    actual: kinds.len() as u16,
                },
            ));
        }
        if let Some(tile) = kinds.iter().copied().find(|kind| kind.is_flower()) {
            return Err(GameError::InvalidHandReplacement(
                MahjongHandReplacementError::FlowerNotAllowed { tile },
            ));
        }
        let mut checked = HashSet::new();
        for tile in kinds.iter().copied() {
            if !checked.insert(tile) {
                continue;
            }
            let requested = kinds.iter().filter(|kind| **kind == tile).count();
            let available = old_hand
                .iter()
                .chain(self.wall.iter())
                .filter(|candidate| candidate.kind() == tile)
                .count();
            if requested > available {
                return Err(GameError::InvalidHandReplacement(
                    MahjongHandReplacementError::TileUnavailable {
                        tile,
                        requested: requested as u16,
                        available: available as u16,
                    },
                ));
            }
        }

        let mut old_used = vec![false; old_hand.len()];
        let mut replacement = vec![None; kinds.len()];
        let mut missing = Vec::new();
        for (desired_index, kind) in kinds.iter().copied().enumerate() {
            if let Some((old_index, tile)) = old_hand
                .iter()
                .copied()
                .enumerate()
                .find(|(index, tile)| !old_used[*index] && tile.kind() == kind)
            {
                old_used[old_index] = true;
                replacement[desired_index] = Some(tile);
            } else {
                missing.push((desired_index, kind));
            }
        }

        let returned = old_hand
            .iter()
            .copied()
            .enumerate()
            .filter_map(|(index, tile)| (!old_used[index]).then_some(tile))
            .collect::<Vec<_>>();
        let mut wall_used = vec![false; self.wall.len()];
        let mut swaps = Vec::with_capacity(missing.len());
        for ((desired_index, kind), returned_tile) in
            missing.into_iter().zip(returned.iter().copied())
        {
            let Some((wall_index, wall_tile)) = self
                .wall
                .iter()
                .copied()
                .enumerate()
                .find(|(index, tile)| !wall_used[*index] && tile.kind() == kind)
            else {
                return Err(GameError::InvalidHandReplacement(
                    MahjongHandReplacementError::TileUnavailable {
                        tile: kind,
                        requested: kinds.iter().filter(|candidate| **candidate == kind).count()
                            as u16,
                        available: old_hand
                            .iter()
                            .chain(self.wall.iter())
                            .filter(|candidate| candidate.kind() == kind)
                            .count() as u16,
                    },
                ));
            };
            wall_used[wall_index] = true;
            replacement[desired_index] = Some(wall_tile);
            swaps.push((wall_index, returned_tile));
        }

        let new_hand = replacement
            .into_iter()
            .collect::<Option<Vec<_>>>()
            .expect("validated replacement fills every hand position");
        for (wall_index, returned_tile) in swaps {
            self.wall[wall_index] = returned_tile;
        }
        if player == self.current_player && self.last_drawn.is_some() {
            self.last_drawn = new_hand.last().copied();
        }
        self.players[player.0].hand = new_hand;
        Ok(())
    }
}
