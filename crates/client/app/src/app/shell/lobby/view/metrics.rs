use crate::app::games::game_seat_count;
use leocard_protocol::LobbySnapshot;

pub(super) struct LobbyMetrics<'a> {
    lobby: &'a LobbySnapshot,
}

impl<'a> LobbyMetrics<'a> {
    pub(super) fn new(lobby: &'a LobbySnapshot) -> Self {
        Self { lobby }
    }

    pub(super) fn connected_player_count(&self) -> usize {
        self.lobby
            .players
            .iter()
            .filter(|player| player.connected)
            .count()
    }

    pub(super) fn ready_player_count(&self) -> usize {
        self.lobby
            .players
            .iter()
            .filter(|player| player.connected && player.ready)
            .count()
    }

    pub(super) fn seat_count(&self) -> u8 {
        game_seat_count(&self.lobby.rules)
    }
}
