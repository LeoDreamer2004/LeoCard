use crate::{ConnectionId, Delivery, RoomSession};
use leocard_protocol::{
    ChatContent, ClientCommand, ClientMessage, GameCommand, GameKind, GameRules, GameSnapshot,
    GameViolation, JoinRequest, LobbySnapshot, PlayerId, PlayerInteractionKind, PlayerViolation,
    RejectReason, RequestId, SeatId, ServerEvent,
};

pub(super) trait HostedGameLifecycle: Sized {
    const KIND: GameKind;

    fn room(&self) -> &RoomSession;

    fn room_mut(&mut self) -> &mut RoomSession;

    fn game_started(&self) -> bool;

    fn capacity(&self) -> u8;

    fn game_rules(&self) -> GameRules;

    fn game_snapshot(&self, recipient: PlayerId) -> GameSnapshot;

    fn handle_game_command(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        command: GameCommand,
    ) -> Result<Vec<Delivery>, GameKind>;

    fn start_game(&mut self, connection: ConnectionId, request_id: RequestId) -> Vec<Delivery>;

    fn return_to_lobby(&mut self, connection: ConnectionId, request_id: RequestId)
    -> Vec<Delivery>;

    fn play_again(&mut self, connection: ConnectionId, request_id: RequestId) -> Vec<Delivery>;

    fn leave_room(&mut self, connection: ConnectionId, request_id: RequestId) -> Vec<Delivery>;

    fn close_room(&mut self, connection: ConnectionId, request_id: RequestId) -> Vec<Delivery> {
        self.room_mut()
            .close_room(connection, request_id)
            .unwrap_or_else(|reason| self.room().reject(connection, request_id, reason))
    }
    fn interact(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        target: PlayerId,
        kind: PlayerInteractionKind,
    ) -> Vec<Delivery>;

    fn before_dispatch(&mut self) {}

    fn after_join(&mut self, _player: PlayerId) {}

    fn join(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        request: JoinRequest,
    ) -> Vec<Delivery> {
        let game_started = self.game_started();
        let capacity = self.capacity();
        let joined = self
            .room_mut()
            .join(connection, request_id, request, game_started, capacity);
        match joined {
            Ok(mut deliveries) => {
                let player = self
                    .room()
                    .player_id(connection)
                    .expect("a successful join assigned a player");
                self.after_join(player);
                deliveries.extend(if game_started {
                    self.broadcast_game(Some((connection, request_id)))
                } else {
                    self.broadcast_lobby(Some((connection, request_id)))
                });
                deliveries
            }
            Err(reason) => self.room().reject(connection, request_id, reason),
        }
    }

    fn set_avatar(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        png: Vec<u8>,
    ) -> Vec<Delivery> {
        let game_started = self.game_started();
        match self.room_mut().set_avatar(connection, png, game_started) {
            Ok(mut deliveries) => {
                deliveries.extend(self.broadcast_lobby(Some((connection, request_id))));
                deliveries
            }
            Err(reason) => self.room().reject(connection, request_id, reason),
        }
    }

    fn select_seat(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        seat: SeatId,
    ) -> Vec<Delivery> {
        let game_started = self.game_started();
        match self.room_mut().select_seat(connection, seat, game_started) {
            Ok(()) => self.broadcast_lobby(Some((connection, request_id))),
            Err(reason) => self.room().reject(connection, request_id, reason),
        }
    }

    fn configure_bot_seat(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        seat: SeatId,
        occupied: bool,
    ) -> Vec<Delivery> {
        let game_started = self.game_started();
        match self
            .room_mut()
            .configure_bot_seat(connection, seat, occupied, game_started)
        {
            Ok(()) => self.broadcast_lobby(Some((connection, request_id))),
            Err(reason) => self.room().reject(connection, request_id, reason),
        }
    }

    fn set_ready(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        ready: bool,
    ) -> Vec<Delivery> {
        let game_started = self.game_started();
        match self.room_mut().set_ready(connection, ready, game_started) {
            Ok(()) => self.broadcast_lobby(Some((connection, request_id))),
            Err(reason) => self.room().reject(connection, request_id, reason),
        }
    }

    fn chat(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        content: ChatContent,
    ) -> Vec<Delivery> {
        let game_started = self.game_started();
        match self
            .room_mut()
            .chat(connection, request_id, content, game_started)
        {
            Ok(deliveries) => deliveries,
            Err(reason) => self.room().reject(connection, request_id, reason),
        }
    }

    fn broadcast_lobby(&self, origin: Option<(ConnectionId, RequestId)>) -> Vec<Delivery> {
        self.room()
            .broadcast_lobby(Self::KIND, self.game_rules(), origin)
    }

    fn lobby_snapshot(&self) -> LobbySnapshot {
        self.room().lobby_snapshot(Self::KIND, self.game_rules())
    }

    fn broadcast_game(&self, origin: Option<(ConnectionId, RequestId)>) -> Vec<Delivery> {
        assert!(
            self.game_started(),
            "game broadcast requires an active game"
        );
        self.room().broadcast_private(origin, |player| {
            ServerEvent::GameSnapshot(self.game_snapshot(player.id))
        })
    }

    fn snapshot(&self, connection: ConnectionId, request_id: RequestId) -> Vec<Delivery> {
        let Some(player) = self.room().player_id(connection) else {
            return self.room().reject(
                connection,
                request_id,
                RejectReason::Player(PlayerViolation::NotJoined),
            );
        };
        let event = if self.game_started() {
            ServerEvent::GameSnapshot(self.game_snapshot(player))
        } else {
            ServerEvent::LobbySnapshot(self.lobby_snapshot())
        };
        vec![self.room().delivery(connection, Some(request_id), event)]
    }
}

pub(super) fn dispatch_client_command<S: HostedGameLifecycle>(
    session: &mut S,
    connection: ConnectionId,
    message: ClientMessage,
) -> Vec<Delivery> {
    if let Err(deliveries) = session.room_mut().begin_request(connection, &message) {
        return deliveries;
    }
    session.before_dispatch();
    let request_id = message.request_id;
    match message.command {
        ClientCommand::Join(request) => session.join(connection, request_id, *request),
        ClientCommand::SetAvatar { png } => session.set_avatar(connection, request_id, png),
        ClientCommand::SelectSeat { seat } => session.select_seat(connection, request_id, seat),
        ClientCommand::ConfigureBotSeat { seat, occupied } => {
            session.configure_bot_seat(connection, request_id, seat, occupied)
        }
        ClientCommand::SetReady { ready } => session.set_ready(connection, request_id, ready),
        ClientCommand::Game(command) => {
            let received = command.kind();
            session
                .handle_game_command(connection, request_id, command)
                .unwrap_or_else(|_| {
                    session.room().reject(
                        connection,
                        request_id,
                        RejectReason::Game(GameViolation::WrongGame {
                            expected: S::KIND,
                            received,
                        }),
                    )
                })
        }
        ClientCommand::StartGame => session.start_game(connection, request_id),
        ClientCommand::ReturnToLobby => session.return_to_lobby(connection, request_id),
        ClientCommand::PlayAgain => session.play_again(connection, request_id),
        ClientCommand::LeaveRoom => session.leave_room(connection, request_id),
        ClientCommand::CloseRoom => session.close_room(connection, request_id),
        ClientCommand::Interact { target, kind } => {
            session.interact(connection, request_id, target, kind)
        }
        ClientCommand::Chat { content } => session.chat(connection, request_id, content),
        ClientCommand::RequestSnapshot => session.snapshot(connection, request_id),
        ClientCommand::Ping => unreachable!("transport pings are handled by HostSession"),
    }
}
