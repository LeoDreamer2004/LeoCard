use super::RoomSession;
use crate::{ConnectionId, Delivery};
use leocard_protocol::{
    PlayerViolation, RejectReason, RequestId, RoomModerationSnapshot, RoomParticipant,
    RoomViolation, ServerEvent,
};

impl RoomSession {
    pub(crate) fn moderation_snapshot(&self) -> RoomModerationSnapshot {
        RoomModerationSnapshot {
            host: self.host_player_id(),
            spectator_interactions_blocked: self.spectator_interactions_blocked,
            participants: self
                .participants()
                .filter(|player| !player.left)
                .map(|player| RoomParticipant {
                    player: player.public_metadata().player,
                    spectating: self.is_spectator(player.connection),
                    connected: player.connected,
                    is_bot: player.is_bot,
                })
                .collect(),
        }
    }

    pub(crate) fn broadcast_moderation(
        &self,
        origin: Option<(ConnectionId, RequestId)>,
    ) -> Vec<Delivery> {
        self.broadcast_event(
            origin,
            ServerEvent::RoomModerationSnapshot(self.moderation_snapshot()),
        )
    }

    pub(crate) fn set_spectator_interactions_blocked(
        &mut self,
        connection: ConnectionId,
        blocked: bool,
    ) -> Result<(), RejectReason> {
        self.player_id(connection)
            .ok_or(RejectReason::Player(PlayerViolation::NotJoined))?;
        if self.host_connection != Some(connection) {
            return Err(RejectReason::Room(RoomViolation::OnlyHostCanConfigure));
        }
        if self.spectator_interactions_blocked != blocked {
            self.spectator_interactions_blocked = blocked;
            self.bump_revision();
        }
        Ok(())
    }

    pub(crate) fn check_interaction_permission(
        &self,
        connection: ConnectionId,
    ) -> Result<(), RejectReason> {
        if self.spectator_interactions_blocked && self.is_spectator(connection) {
            return Err(RejectReason::Room(
                RoomViolation::SpectatorInteractionsBlocked,
            ));
        }
        Ok(())
    }
}
