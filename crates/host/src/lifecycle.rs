use crate::{ConnectionId, Delivery, RoomSession};
use leocard_protocol::{
    ChatContent, ClientCommand, ClientMessage, GameCommand, GameEventContext, GameKind, GameRules,
    GameSnapshot, GameViolation, JoinRequest, LobbySnapshot, PlayerId, PlayerInteraction,
    PlayerInteractionKind, PlayerViolation, RejectReason, RequestId, RoomInfo, RoomViolation,
    SeatId, ServerEvent,
};

pub(super) trait HostedGameLifecycle: Sized {
    const KIND: GameKind;

    fn room(&self) -> &RoomSession;

    fn room_mut(&mut self) -> &mut RoomSession;

    fn game_started(&self) -> bool;

    fn capacity(&self) -> u8;

    fn game_rules(&self) -> GameRules;

    fn game_snapshot(&self, recipient: PlayerId) -> GameSnapshot;

    fn room_info(&self) -> Option<RoomInfo> {
        let room = self.room();
        let host = room
            .players
            .iter()
            .find(|player| Some(player.connection) == room.host_connection && !player.left)?;
        (!room.closed).then(|| RoomInfo {
            host_name: host.name.clone(),
            host_avatar: host.avatar,
            avatar_png: host.avatar_png.clone(),
            game: Self::KIND,
            playing: self.game_started(),
        })
    }

    fn handle_game_command(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        command: GameCommand,
    ) -> Result<Vec<Delivery>, GameKind>;

    fn start_game(&mut self, connection: ConnectionId, request_id: RequestId) -> Vec<Delivery>;

    fn start_game_and_announce(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
    ) -> Vec<Delivery> {
        let was_started = self.game_started();
        let mut deliveries = self.start_game(connection, request_id);
        if !was_started && self.game_started() {
            let host = self
                .room()
                .host_player_id()
                .expect("starting a game requires a host");
            let match_id = self.game_snapshot(host).match_id();
            self.room_mut().bump_revision();
            deliveries.extend(self.with_game_context(self.room().broadcast_event(
                None,
                ServerEvent::GameStarted {
                    host,
                    game: Self::KIND,
                    match_id,
                },
            )));
        }
        deliveries
    }

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
    ) -> Vec<Delivery> {
        let result = self
            .room()
            .player_id(connection)
            .ok_or(RejectReason::Player(PlayerViolation::NotJoined))
            .and_then(|source| {
                if source == target {
                    return Err(RejectReason::Room(RoomViolation::CannotInteractWithSelf));
                }
                if !self
                    .room()
                    .players
                    .iter()
                    .any(|player| player.id == target && !player.left)
                {
                    return Err(RejectReason::Player(PlayerViolation::NotJoined));
                }
                Ok(self
                    .room_mut()
                    .record_interaction(source, target, kind, fastrand::u32(..)))
            });
        match result {
            Ok(interaction) => {
                self.observe_interaction(&interaction);
                self.room_mut().bump_revision();
                let mut deliveries = self.room().broadcast_event(
                    Some((connection, request_id)),
                    ServerEvent::PlayerInteraction(interaction),
                );
                deliveries.extend(if self.game_started() {
                    self.broadcast_game(None)
                } else {
                    self.broadcast_lobby(None)
                });
                self.with_game_context(deliveries)
            }
            Err(reason) => self.room().reject(connection, request_id, reason),
        }
    }

    fn before_dispatch(&mut self) {}

    fn observe_interaction(&mut self, _interaction: &PlayerInteraction) {}

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
                if !self.room().is_spectator(connection) {
                    self.after_join(player);
                }
                if game_started {
                    deliveries.push(self.room().delivery(
                        connection,
                        None,
                        ServerEvent::RulesSnapshot(self.game_rules()),
                    ));
                }
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
        match self.room_mut().set_avatar(connection, png) {
            Ok(mut deliveries) => {
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
            Ok(deliveries) => self.with_game_context(deliveries),
            Err(reason) => self.room().reject(connection, request_id, reason),
        }
    }

    fn with_game_context(&self, mut deliveries: Vec<Delivery>) -> Vec<Delivery> {
        if self.game_started()
            && let Some(host) = self.room().host_player_id()
        {
            let context = GameEventContext {
                records_progress: self.room().records_game_progress(),
                match_id: self.game_snapshot(host).match_id(),
                hand_index: None,
                sequence: u128::from(self.room().revision.0) << 64,
            };
            for delivery in &mut deliveries {
                delivery.message.game_context = Some(context);
            }
        }
        deliveries
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
    if session.room().is_spectator(connection)
        && !matches!(
            &message.command,
            ClientCommand::RequestSnapshot
                | ClientCommand::LeaveRoom
                | ClientCommand::SetAvatar { .. }
                | ClientCommand::PublishCoins { .. }
                | ClientCommand::PublishAchievements { .. }
        )
    {
        return session.room().reject(
            connection,
            message.request_id,
            RejectReason::Room(RoomViolation::SpectatorCannotAct),
        );
    }
    let mut deliveries = dispatch_joined_command(session, connection, message);
    if !session.room().closed && !session.game_started() && session.room().has_spectators() {
        let capacity = session.capacity();
        let admissions = session.room_mut().admit_spectators(capacity);
        if !admissions.is_empty() {
            deliveries.retain(|delivery| {
                !matches!(delivery.message.event, ServerEvent::LobbySnapshot(_))
            });
            deliveries.extend(admissions);
            deliveries.extend(session.broadcast_lobby(None));
        }
    }
    deliveries
}

fn dispatch_joined_command<S: HostedGameLifecycle>(
    session: &mut S,
    connection: ConnectionId,
    message: ClientMessage,
) -> Vec<Delivery> {
    if session.room().is_spectator(connection)
        && matches!(message.command, ClientCommand::LeaveRoom)
    {
        session.room_mut().remove_spectator(connection);
        return vec![session.room().delivery(
            connection,
            Some(message.request_id),
            ServerEvent::LeftRoom,
        )];
    }
    session.before_dispatch();
    let request_id = message.request_id;
    match message.command {
        ClientCommand::PublishCoins { coins } => {
            match session.room_mut().publish_coins(connection, coins) {
                Ok(()) => {
                    if session.game_started() {
                        session.broadcast_game(Some((connection, request_id)))
                    } else {
                        session.broadcast_lobby(Some((connection, request_id)))
                    }
                }
                Err(reason) => session.room().reject(connection, request_id, reason),
            }
        }
        ClientCommand::PublishAchievements { counts, unlocked } => {
            match session
                .room_mut()
                .publish_achievements(connection, request_id, counts, unlocked)
            {
                Ok(mut deliveries) => {
                    deliveries.extend(if session.game_started() {
                        session.broadcast_game(Some((connection, request_id)))
                    } else {
                        session.broadcast_lobby(Some((connection, request_id)))
                    });
                    deliveries
                }
                Err(reason) => session.room().reject(connection, request_id, reason),
            }
        }
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
        ClientCommand::StartGame => session.start_game_and_announce(connection, request_id),
        ClientCommand::ReturnToLobby => session.return_to_lobby(connection, request_id),
        ClientCommand::PlayAgain => {
            if session.room().has_spectators()
                && session.room().host_connection == Some(connection)
                && session.game_started()
                && session
                    .game_snapshot(session.room().host_player_id().expect("room host"))
                    .match_complete()
            {
                session.return_to_lobby(connection, request_id)
            } else {
                session.play_again(connection, request_id)
            }
        }
        ClientCommand::LeaveRoom => session.leave_room(connection, request_id),
        ClientCommand::CloseRoom => session.close_room(connection, request_id),
        ClientCommand::Interact { target, kind } => {
            session.interact(connection, request_id, target, kind)
        }
        ClientCommand::Chat { content } => session.chat(connection, request_id, content),
        ClientCommand::RequestSnapshot => session.snapshot(connection, request_id),
        ClientCommand::RequestRoomInfo => unreachable!("discovery is handled by HostSession"),
        ClientCommand::Ping => unreachable!("transport pings are handled by HostSession"),
    }
}
