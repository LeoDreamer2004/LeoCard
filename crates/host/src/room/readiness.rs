use super::RoomSession;
use crate::ConnectionId;
use leocard_protocol::{GameViolation, PlayerViolation, RejectReason, RoomViolation};

impl RoomSession {
    /// Discard the finished match's absent participants without renumbering online players.
    /// Lobby snapshots do not replace the recipient's in-game PlayerId.
    pub(crate) fn prepare_lobby(&mut self) {
        self.remove_bots();
        for player in &mut self.players {
            player.auto_play = false;
            if !player.connected {
                player.left = true;
                player.seat = None;
            }
            player.ready = !player.left && self.host_connection == Some(player.connection);
        }
    }

    pub(crate) fn validate_start(
        &self,
        connection: ConnectionId,
        minimum: u8,
        started: bool,
    ) -> Result<(), RejectReason> {
        if self.player_id(connection).is_none() {
            return Err(RejectReason::Player(PlayerViolation::NotJoined));
        }
        if started {
            return Err(RejectReason::Game(GameViolation::GameAlreadyStarted));
        }
        if self.host_connection != Some(connection) {
            return Err(RejectReason::Room(RoomViolation::OnlyHostCanStart));
        }
        let players: Vec<_> = self.players.iter().filter(|player| !player.left).collect();
        if players.len() < usize::from(minimum) {
            return Err(RejectReason::Room(RoomViolation::NotEnoughPlayers {
                minimum,
                actual: players.len() as u8,
            }));
        }
        if players.iter().any(|player| player.seat.is_none()) {
            return Err(RejectReason::Room(RoomViolation::MustSelectSeat));
        }
        let unready: Vec<_> = players
            .iter()
            .filter(|player| !player.ready || (!player.connected && !player.is_bot))
            .map(|player| player.id)
            .collect();
        if !unready.is_empty() {
            return Err(RejectReason::Room(RoomViolation::PlayersNotReady {
                players: unready,
            }));
        }
        Ok(())
    }
}
