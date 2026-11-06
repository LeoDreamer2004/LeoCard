use super::GuandanSession;
use crate::lifecycle::HostedGameLifecycle;
use crate::{ConnectionId, Delivery, RoomSession};
use leocard_game_common::PlayerId;
use leocard_guandan::{GuandanPhase, build_deck};
use leocard_protocol::{
    GameKind, GameRules, GameViolation, PlayerViolation, RejectReason, RequestId, RoomViolation,
    ServerEvent,
};

impl HostedGameLifecycle for GuandanSession {
    const KIND: GameKind = GameKind::Guandan;

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
        4
    }

    fn game_rules(&self) -> GameRules {
        self.rules.into()
    }

    fn start_game(&mut self, connection: ConnectionId, request: RequestId) -> Vec<Delivery> {
        if let Err(reason) = self.room.validate_start(connection, 4, self.game.is_some()) {
            return self.room.reject(connection, request, reason);
        }
        let count = self
            .room
            .players
            .iter()
            .filter(|player| !player.left)
            .count();
        if count != 4 {
            return self.room.reject(
                connection,
                request,
                RejectReason::Room(RoomViolation::WaitingForPlayers {
                    expected: 4,
                    actual: count as u8,
                }),
            );
        }
        self.room.remove_departed_players();
        self.room
            .players
            .sort_by_key(|player| player.seat.expect("all seated").0);
        for (index, player) in self.room.players.iter_mut().enumerate() {
            player.id = PlayerId(index as u8);
        }
        self.begin_match();
        self.room.bump_revision();
        self.broadcast_game(Some((connection, request)))
    }

    fn return_to_lobby(&mut self, connection: ConnectionId, request: RequestId) -> Vec<Delivery> {
        if self.room.host_connection != Some(connection) {
            return self.room.reject(
                connection,
                request,
                RejectReason::Room(RoomViolation::OnlyHostCanReturnToLobby),
            );
        }
        if self
            .match_state
            .as_ref()
            .is_none_or(|state| !state.complete())
        {
            return self.room.reject(
                connection,
                request,
                RejectReason::Game(GameViolation::GameNotFinished),
            );
        }
        self.game = None;
        self.match_id = None;
        self.match_state = None;
        self.finished_reference_changes = None;
        self.room.prepare_lobby();
        let mut deck = build_deck();
        fastrand::shuffle(&mut deck);
        self.deck = Some(deck);
        self.room.bump_revision();
        self.broadcast_lobby(Some((connection, request)))
    }

    fn play_again(&mut self, connection: ConnectionId, request: RequestId) -> Vec<Delivery> {
        if self
            .game
            .as_ref()
            .is_none_or(|game| !matches!(game.phase(), GuandanPhase::Finished(_)))
        {
            return self.room.reject(
                connection,
                request,
                RejectReason::Game(GameViolation::GameNotFinished),
            );
        }
        let Some(player) = self.room.player_id(connection) else {
            return self.room.reject(
                connection,
                request,
                RejectReason::Player(PlayerViolation::NotJoined),
            );
        };
        self.room.players[player.index()].ready = true;
        if self.everyone_ready() {
            self.continue_match();
        }
        self.room.bump_revision();
        self.broadcast_game(Some((connection, request)))
    }

    fn leave_room(&mut self, connection: ConnectionId, request: RequestId) -> Vec<Delivery> {
        if self.room.host_connection == Some(connection) {
            return self.close_room(connection, request);
        }
        let Some(player) = self
            .room
            .players
            .iter_mut()
            .find(|player| player.connection == connection && !player.left)
        else {
            return self.room.reject(
                connection,
                request,
                RejectReason::Player(PlayerViolation::NotJoined),
            );
        };
        let name = player.name.clone();
        player.left = true;
        player.connected = false;
        player.ready = false;
        if self.game.is_none() {
            player.seat = None;
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
