use super::DrawGuessSession;
use crate::lifecycle::HostedGameLifecycle;
use crate::{ConnectionId, Delivery, RoomSession};
use leocard_draw_guess::DrawGuessRuleSet;
use leocard_game_common::PlayerId;
use leocard_protocol::{GameKind, GameRules, RejectReason, RequestId, RoomViolation, ServerEvent};

impl HostedGameLifecycle for DrawGuessSession {
    const KIND: GameKind = GameKind::DrawGuess;

    fn room(&self) -> &RoomSession {
        &self.room
    }

    fn room_mut(&mut self) -> &mut RoomSession {
        &mut self.room
    }

    fn game_started(&self) -> bool {
        self.game.is_some()
    }

    fn capacity(&self) -> u8 {
        DrawGuessRuleSet::MAX_PLAYERS
    }

    fn lobby_state_event(&self, recipient: PlayerId) -> Option<ServerEvent> {
        Some(self.library_state(recipient))
    }

    fn game_rules(&self) -> GameRules {
        self.rules.into()
    }

    fn after_join(&mut self, player: PlayerId) {
        if let Some(game) = &mut self.game {
            game.set_connected(player, true);
        }
    }

    fn viewing_player(&self, recipient: PlayerId) -> PlayerId {
        // Spectators never inherit the artist's private choices or answer.
        recipient
    }

    fn start_game(&mut self, connection: ConnectionId, request: RequestId) -> Vec<Delivery> {
        if let Err(reason) = self.room.validate_start(
            connection,
            DrawGuessRuleSet::MIN_PLAYERS,
            self.game.is_some(),
        ) {
            return self.reject(connection, request, reason);
        }
        self.begin_match();
        self.broadcast_game(Some((connection, request)))
    }

    fn return_to_lobby(&mut self, connection: ConnectionId, request: RequestId) -> Vec<Delivery> {
        if let Err(reason) = self.require_finished(connection) {
            return self.reject(connection, request, reason);
        }
        if self.room.host_connection != Some(connection) {
            return self.reject(
                connection,
                request,
                RejectReason::Room(RoomViolation::OnlyHostCanReturnToLobby),
            );
        }
        self.game = None;
        self.match_id = None;
        self.finished_reference_changes = None;
        self.order.clear();
        self.room.prepare_lobby();
        self.room.bump_revision();
        self.broadcast_lobby(Some((connection, request)))
    }

    fn play_again(&mut self, connection: ConnectionId, request: RequestId) -> Vec<Delivery> {
        let player = match self.require_finished(connection) {
            Ok(player) => player,
            Err(reason) => return self.reject(connection, request, reason),
        };
        self.room
            .players
            .iter_mut()
            .find(|participant| participant.id == player)
            .expect("joined player")
            .ready = true;
        if self.everyone_ready() {
            self.begin_match();
        }
        self.room.bump_revision();
        self.broadcast_game(Some((connection, request)))
    }

    fn leave_room(&mut self, connection: ConnectionId, request: RequestId) -> Vec<Delivery> {
        let player = match self.require_player(connection) {
            Ok(player) => player,
            Err(reason) => return self.reject(connection, request, reason),
        };
        if self.room.host_connection == Some(connection) {
            return self.close_room(connection, request);
        }
        let participant = self
            .room
            .players
            .iter_mut()
            .find(|participant| participant.id == player)
            .expect("joined player");
        let name = participant.name.clone();
        participant.left = true;
        participant.connected = false;
        participant.ready = false;
        if self.game.is_none() {
            participant.seat = None;
        }
        if let Some(game) = &mut self.game {
            game.set_connected(player, false);
        }
        self.room.bump_revision();
        let mut deliveries =
            vec![
                self.room
                    .delivery(connection, Some(request), ServerEvent::LeftRoom),
            ];
        deliveries.extend(
            self.room
                .broadcast_event(None, ServerEvent::PlayerLeft { name }),
        );
        deliveries.extend(if self.game.is_some() {
            self.broadcast_game(None)
        } else {
            self.broadcast_lobby(None)
        });
        deliveries
    }
}
