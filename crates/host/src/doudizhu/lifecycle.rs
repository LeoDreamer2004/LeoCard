use super::DoudizhuSession;
use crate::lifecycle::HostedGameLifecycle;
use crate::{ConnectionId, Delivery, RoomSession};
use leocard_doudizhu::{DoudizhuPhase, build_deck};
use leocard_game_common::PlayerId;
use leocard_protocol::{
    GameKind, GameRules, GameViolation, PlayerInteraction, PlayerInteractionKind, PlayerViolation,
    RejectReason, RequestId, RoomViolation, ServerEvent,
};

impl HostedGameLifecycle for DoudizhuSession {
    const KIND: GameKind = GameKind::Doudizhu;

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
        self.rules.mode.players()
    }

    fn game_rules(&self) -> GameRules {
        self.rules.into()
    }

    fn observe_interaction(&mut self, interaction: &PlayerInteraction) {
        if interaction.kind == PlayerInteractionKind::Flower
            && let Some(game) = &self.game
        {
            self.achievements
                .observe_flower(interaction.source, interaction.target, game);
        }
    }

    fn start_game(&mut self, connection: ConnectionId, request: RequestId) -> Vec<Delivery> {
        if let Err(reason) =
            self.room
                .validate_start(connection, self.rules.mode.players(), self.game.is_some())
        {
            return self.room.reject(connection, request, reason);
        }
        let count = self.room.players.iter().filter(|p| !p.left).count();
        let seats = self.rules.mode.players();
        if count != usize::from(seats) {
            return self.room.reject(
                connection,
                request,
                RejectReason::Room(RoomViolation::WaitingForPlayers {
                    expected: seats,
                    actual: count as u8,
                }),
            );
        }
        self.room.remove_departed_players();
        // Seat numbers run clockwise while the rules advance through player IDs.
        self.room
            .players
            .sort_by_key(|p| (seats - p.seat.expect("all seated").0) % seats);
        for (i, p) in self.room.players.iter_mut().enumerate() {
            p.id = PlayerId(i as u8);
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
        self.deck = None;
        self.room.prepare_lobby();
        let mut deck = build_deck(self.rules.mode);
        fastrand::shuffle(&mut deck);
        self.deck = Some(deck);
        self.room.bump_revision();
        self.broadcast_lobby(Some((connection, request)))
    }

    fn play_again(&mut self, connection: ConnectionId, request: RequestId) -> Vec<Delivery> {
        if self
            .game
            .as_ref()
            .is_none_or(|g| g.phase != DoudizhuPhase::Finished)
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
        self.room.players[usize::from(player.0)].ready = true;
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
        let Some(p) = self
            .room
            .players
            .iter_mut()
            .find(|p| p.connection == connection && !p.left)
        else {
            return self.room.reject(
                connection,
                request,
                RejectReason::Player(PlayerViolation::NotJoined),
            );
        };
        let name = p.name.clone();
        p.left = true;
        p.connected = false;
        p.ready = false;
        if self.game.is_none() {
            p.seat = None;
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
