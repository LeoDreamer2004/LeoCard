use super::super::MinecraftGuessSession;
use crate::lifecycle::HostedGameLifecycle;
use crate::{ConnectionId, Delivery};
use leocard_protocol::{GameViolation, MinecraftGuessEvent, RejectReason, RequestId};

impl MinecraftGuessSession {
    pub(in super::super) fn report_minecraft_process(
        &mut self,
        connection: ConnectionId,
        request: RequestId,
        running: bool,
    ) -> Vec<Delivery> {
        let player = match self.require_player(connection) {
            Ok(player) => player,
            Err(reason) => return self.reject(connection, request, reason),
        };
        if self.game.is_none() {
            return self.reject(
                connection,
                request,
                RejectReason::Game(GameViolation::GameNotStarted),
            );
        }
        if !self.process_presence.update(player, running) {
            return self.snapshot(connection, request);
        }
        let name = self.room.players[usize::from(player.0)].name.clone();
        self.bump_revision();
        let mut deliveries = self.room.broadcast_game_events(
            [MinecraftGuessEvent::ProcessPresenceChanged {
                player,
                name,
                running,
            }],
            self.match_id.map(|id| (id, None)),
        );
        deliveries.extend(self.broadcast_game(Some((connection, request))));
        deliveries
    }
}
