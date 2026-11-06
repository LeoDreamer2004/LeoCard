use crate::{ConnectionId, Delivery, RoomSession};
use leocard_game_common::PlayerId;
use leocard_protocol::{
    ClientCommand, ClientMessage, PlayerGameProfiles, RequestId, Revision, ServerEvent,
};

pub(crate) struct ActionTimingRequest {
    connection: ConnectionId,
    request: RequestId,
    revision: Revision,
    player: Option<PlayerId>,
    joining: bool,
}

impl ActionTimingRequest {
    pub(crate) fn new(
        room: &RoomSession,
        connection: ConnectionId,
        message: &ClientMessage,
        is_decision: bool,
    ) -> Self {
        let player = is_decision
            .then(|| {
                room.players
                    .iter()
                    .find(|player| {
                        player.connection == connection
                            && player.connected
                            && !player.left
                            && !player.is_bot
                            && !player.auto_play
                    })
                    .map(|player| player.id)
            })
            .flatten();
        Self {
            connection,
            request: message.request_id,
            revision: room.revision,
            player,
            joining: matches!(&message.command, ClientCommand::Join(_)),
        }
    }

    pub(crate) fn complete<'a>(
        self,
        room: &'a mut RoomSession,
        deliveries: &[Delivery],
    ) -> Option<(&'a mut PlayerGameProfiles, u64)> {
        let rejected = deliveries.iter().any(|delivery| {
            delivery.recipient == self.connection
                && delivery.message.in_reply_to == Some(self.request)
                && matches!(delivery.message.event, ServerEvent::Rejected { .. })
        });
        if room.revision == self.revision || rejected {
            return None;
        }
        if self.joining
            && let Some(player) = room.player_id(self.connection)
        {
            // Reconnection discards the previous window even before the next host tick.
            room.action_timing.complete(player);
        }
        let player = self.player?;
        let millis = room.action_timing.complete(player)?;
        let participant = room
            .players
            .iter_mut()
            .find(|participant| participant.id == player)
            .expect("joined decision player");
        Some((&mut participant.game_profiles, millis))
    }
}
