use crate::app::games::{game_accepts_player_count, game_seat_count};
use leocard_protocol::{GameRules, LobbySnapshot};

pub(super) struct LobbyMetrics<'a> {
    lobby: &'a LobbySnapshot,
}

impl<'a> LobbyMetrics<'a> {
    pub(super) fn new(lobby: &'a LobbySnapshot) -> Self {
        Self { lobby }
    }

    pub(super) fn connected_player_count(&self) -> u8 {
        self.lobby
            .players
            .iter()
            .filter(|player| player.connected)
            .count() as u8
    }

    pub(super) fn ready_player_count(&self) -> u8 {
        self.lobby
            .players
            .iter()
            .filter(|player| player.connected && player.ready)
            .count() as u8
    }

    pub(super) fn can_start(&self) -> bool {
        let require_connected = matches!(
            self.lobby.rules,
            GameRules::Blackjack(_) | GameRules::Doudizhu(_)
        );
        game_accepts_player_count(&self.lobby.rules, self.connected_player_count())
            && self.lobby.players.iter().all(|player| {
                if player.connected {
                    player.ready && player.seat.is_some()
                } else {
                    !require_connected
                }
            })
    }

    pub(super) fn seat_count(&self) -> u8 {
        game_seat_count(&self.lobby.rules)
    }
}
