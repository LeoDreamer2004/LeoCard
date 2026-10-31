use super::RoomSession;
use crate::{ConnectionId, Delivery};
use leocard_game_common::PlayerId;
use leocard_protocol::{
    GameViolation, PlayerViolation, RejectReason, RequestId, RoomViolation, ServerEvent,
};

impl RoomSession {
    pub(crate) fn kick_player(
        &mut self,
        connection: ConnectionId,
        request: RequestId,
        target: PlayerId,
        game_started: bool,
    ) -> Result<Vec<Delivery>, RejectReason> {
        let actor = self
            .player_id(connection)
            .ok_or(RejectReason::Player(PlayerViolation::NotJoined))?;
        if self.host_connection != Some(connection) {
            return Err(RejectReason::Room(RoomViolation::OnlyHostCanKick));
        }
        if game_started {
            return Err(RejectReason::Game(GameViolation::GameAlreadyStarted));
        }
        let player = self
            .players
            .iter_mut()
            .find(|player| player.id == target && !player.left)
            .ok_or(RejectReason::Player(PlayerViolation::NotJoined))?;
        if actor == target || player.is_bot {
            return Err(RejectReason::Room(RoomViolation::CannotKickPlayer));
        }
        let recipient = player.connection;
        let name = player.name.clone();
        player.seat = None;
        player.ready = false;
        player.connected = false;
        player.auto_play = false;
        player.left = true;
        self.bump_revision();
        let mut deliveries = vec![self.delivery(recipient, None, ServerEvent::KickedFromRoom)];
        deliveries.extend(self.broadcast_event(
            Some((connection, request)),
            ServerEvent::PlayerKicked { name },
        ));
        Ok(deliveries)
    }
}
