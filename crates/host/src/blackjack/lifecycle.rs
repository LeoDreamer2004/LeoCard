use super::BlackjackSession;
use crate::lifecycle::HostedGameLifecycle;
use crate::{ConnectionId, Delivery, RoomSession};
use leocard_blackjack::BlackjackRuleSet;
use leocard_game_common::PlayerId;
use leocard_protocol::{
    GameCommand, GameKind, GameRules, GameSnapshot, RejectReason, RequestId, RoomViolation,
    ServerEvent,
};

impl HostedGameLifecycle for BlackjackSession {
    const KIND: GameKind = GameKind::Blackjack;

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
        BlackjackRuleSet::MAX_PLAYERS
    }

    fn game_rules(&self) -> GameRules {
        self.rules.into()
    }

    fn game_snapshot(&self, recipient: PlayerId) -> GameSnapshot {
        self.snapshot_for(recipient).into()
    }

    fn handle_game_command(
        &mut self,
        connection: ConnectionId,
        request: RequestId,
        command: GameCommand,
    ) -> Result<Vec<Delivery>, GameKind> {
        let GameCommand::Blackjack(command) = command else {
            return Err(command.kind());
        };
        Ok(self.command(connection, request, command))
    }

    fn start_game(&mut self, connection: ConnectionId, request: RequestId) -> Vec<Delivery> {
        if let Err(reason) = self.room.validate_start(
            connection,
            BlackjackRuleSet::MIN_PLAYERS,
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
        self.order.clear();
        self.finished_reference_changes = None;
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
            self.continue_match();
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
