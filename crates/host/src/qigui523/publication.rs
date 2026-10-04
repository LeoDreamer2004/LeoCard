use super::QiGui523Session;
use crate::{ConnectionId, Delivery, lifecycle::HostedGameLifecycle};
use leocard_protocol::{QiGui523Event, RequestId, ServerEvent};
use leocard_qigui523::Phase;

impl QiGui523Session {
    pub(super) fn broadcast_game_after_update(
        &mut self,
        origin: Option<(ConnectionId, RequestId)>,
    ) -> Vec<Delivery> {
        self.apply_finished_reference_points();
        let finished = self
            .game
            .as_ref()
            .is_some_and(|game| matches!(game.phase(), Phase::Finished(_)));
        if !finished {
            return self.broadcast_game(origin);
        }

        let departed = self
            .players
            .iter()
            .filter(|player| !player.connected && !player.is_bot && !player.left)
            .map(|player| (player.connection, player.name.clone()))
            .collect::<Vec<_>>();
        if departed.is_empty() {
            return self.broadcast_game(origin);
        }
        for player in &mut self.players {
            if !player.connected && !player.is_bot {
                player.ready = false;
                player.left = true;
            }
        }

        if self
            .host_connection
            .is_some_and(|host| departed.iter().any(|(connection, _)| *connection == host))
        {
            self.closed = true;
            return self.room.broadcast_event(None, ServerEvent::RoomClosed);
        }

        let mut deliveries = Vec::new();
        for (_, name) in departed {
            deliveries.extend(
                self.room
                    .broadcast_event(None, ServerEvent::PlayerLeft { name }),
            );
        }
        deliveries.extend(self.broadcast_game(origin));
        deliveries
    }

    pub(super) fn broadcast_game_after_action(
        &mut self,
        origin: Option<(ConnectionId, RequestId)>,
        events: Vec<QiGui523Event>,
    ) -> Vec<Delivery> {
        let mut deliveries = self.broadcast_action_events(events);
        deliveries.extend(self.broadcast_game_after_update(origin));
        deliveries
    }
}
