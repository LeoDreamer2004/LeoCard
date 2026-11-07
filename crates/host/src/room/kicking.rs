use super::RoomSession;
use crate::ConnectionId;
use leocard_game_common::PlayerId;
use leocard_protocol::{PlayerViolation, RejectReason, RoomViolation};

impl RoomSession {
    pub(crate) fn kick_target(
        &self,
        connection: ConnectionId,
        target: PlayerId,
    ) -> Result<(ConnectionId, String), RejectReason> {
        let actor = self
            .player_id(connection)
            .ok_or(RejectReason::Player(PlayerViolation::NotJoined))?;
        if self.host_connection != Some(connection) {
            return Err(RejectReason::Room(RoomViolation::OnlyHostCanKick));
        }
        let player = self
            .participants()
            .find(|player| player.id == target && !player.left)
            .ok_or(RejectReason::Player(PlayerViolation::NotJoined))?;
        if actor == target || player.is_bot {
            return Err(RejectReason::Room(RoomViolation::CannotKickPlayer));
        }
        Ok((player.connection, player.name.clone()))
    }
}
