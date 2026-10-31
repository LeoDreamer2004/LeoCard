use leocard_game_common::PlayerId;
use std::collections::HashSet;

#[derive(Clone, Debug, Default)]
pub(in super::super) struct McProcessPresence {
    players: HashSet<PlayerId>,
}

impl McProcessPresence {
    pub fn update(&mut self, player: PlayerId, running: bool) -> bool {
        if running {
            self.players.insert(player)
        } else {
            self.players.remove(&player)
        }
    }

    pub fn players(&self) -> Vec<PlayerId> {
        let mut players: Vec<_> = self.players.iter().copied().collect();
        players.sort_by_key(|player| player.0);
        players
    }

    pub fn remove(&mut self, player: PlayerId) {
        self.players.remove(&player);
    }

    pub fn clear(&mut self) {
        self.players.clear();
    }
}
