use super::super::MahjongGameState;
use crate::{MahjongMeld, MahjongMeldKind, MahjongTile, MahjongTileKind, waiting_tile_kinds};
use leocard_game_common::PlayerId;

pub(in crate::game) enum ReadyKong {
    Concealed,
    Added,
    Melded(PlayerId),
}

impl MahjongGameState {
    pub fn concealed_kong_available(&self, player: PlayerId, tile: MahjongTileKind) -> bool {
        self.ensure_playing_turn(player).is_ok()
            && !self.wall.is_empty()
            && !tile.is_flower()
            && self.players[player.index()]
                .hand
                .iter()
                .filter(|held| held.kind() == tile)
                .count()
                == 4
            && self.ready_kong_allowed(player, tile, ReadyKong::Concealed)
    }

    pub fn added_kong_available(&self, player: PlayerId, tile: MahjongTile) -> bool {
        self.ensure_playing_turn(player).is_ok()
            && !self.wall.is_empty()
            && self.players[player.index()].hand.contains(&tile)
            && self.players[player.index()]
                .melds
                .iter()
                .any(|meld| meld.kind() == MahjongMeldKind::Pung && meld.tile() == tile.kind())
            && self.ready_kong_allowed(player, tile.kind(), ReadyKong::Added)
    }

    pub(in crate::game) fn ready_kong_allowed(
        &self,
        player: PlayerId,
        tile: MahjongTileKind,
        kong: ReadyKong,
    ) -> bool {
        let state = &self.players[player.index()];
        let Some(before) = &state.ready_waits else {
            return true;
        };
        let mut concealed = state
            .hand
            .iter()
            .map(|held| held.kind())
            .collect::<Vec<_>>();
        let mut melds = state.melds.clone();
        let (removed, meld) = match kong {
            ReadyKong::Concealed => (4, MahjongMeld::concealed_kong(tile)),
            ReadyKong::Melded(from) => (3, MahjongMeld::melded_kong(tile, from)),
            ReadyKong::Added => {
                let Some(index) = melds
                    .iter()
                    .position(|meld| meld.kind() == MahjongMeldKind::Pung && meld.tile() == tile)
                else {
                    return false;
                };
                let from = melds.remove(index).claimed_from().expect("pung is open");
                (1, MahjongMeld::melded_kong(tile, from))
            }
        };
        for _ in 0..removed {
            let Some(index) = concealed.iter().position(|held| *held == tile) else {
                return false;
            };
            concealed.remove(index);
        }
        melds.push(meld);
        *before == waiting_tile_kinds(&concealed, &melds)
    }
}
