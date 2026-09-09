use crate::{ConnectionId, Delivery};
use leocard_protocol::{
    AvatarId, ChatContent, ChatMessage, ClientMessage, GameEvent, GameKind, GameRules,
    GameViolation, JoinRequest, LobbySnapshot, MAX_CHAT_MESSAGE_CHARS, MAX_PLAYER_NAME_CHARS,
    PROTOCOL_VERSION, PlayerGameProfiles, PlayerId, PlayerInteractionKind, PlayerInteractionStats,
    PlayerViolation, ProfileId, QUICK_VOICE_COUNT, ReconnectToken, RejectReason, RequestId,
    RequestViolation, Revision, RoomId, RoomViolation, SeatId, ServerEvent, ServerMessage,
    TABLE_SEAT_COUNT,
};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub(super) struct Participant {
    pub id: PlayerId,
    pub profile_id: ProfileId,
    pub connection: ConnectionId,
    pub name: String,
    pub avatar: Option<AvatarId>,
    pub avatar_png: Option<Vec<u8>>,
    pub reconnect_token: ReconnectToken,
    pub seat: Option<SeatId>,
    pub ready: bool,
    pub connected: bool,
    /// 两种已接入房主的游戏均使用此字段执行确定性托管策略。
    pub auto_play: bool,
    /// 仅开发者模式手动占座使用；机器人没有真实网络连接。
    pub is_bot: bool,
    pub left: bool,
    pub reference_points: i32,
    pub completed_games: u32,
    pub game_profiles: PlayerGameProfiles,
}

/// 不理解具体棋牌游戏规则的房间状态。
///
/// 游戏后端持有本类型；TCP 连接、身份、头像、座位、准备、修订号和投递格式只在
/// 这里保存一份。具体游戏仍决定何时允许加入、何时广播大厅或私有牌局快照。
#[derive(Clone, Debug)]
pub struct RoomSession {
    pub(super) room_id: RoomId,
    pub(super) host_port: u16,
    pub(super) players: Vec<Participant>,
    pub(super) host_connection: Option<ConnectionId>,
    pub(super) last_requests: HashMap<ConnectionId, RequestId>,
    pub(super) closed: bool,
    pub(super) revision: Revision,
    pub(super) next_avatar_id: u64,
    pub(super) seat_count: u8,
}

impl RoomSession {
    fn active_participants(&self) -> impl Iterator<Item = &Participant> {
        self.players
            .iter()
            .filter(|player| player.connected && !player.left)
    }

    pub(super) fn heartbeat(&self) -> Vec<Delivery> {
        self.broadcast_event(None, ServerEvent::Heartbeat)
    }

    pub(super) fn new(room_id: RoomId, host_port: u16, capacity: usize) -> Self {
        Self::new_with_seat_count(room_id, host_port, capacity, TABLE_SEAT_COUNT)
    }

    pub(super) fn new_with_seat_count(
        room_id: RoomId,
        host_port: u16,
        capacity: usize,
        seat_count: u8,
    ) -> Self {
        assert!(seat_count > 0 && seat_count <= TABLE_SEAT_COUNT);
        Self {
            room_id,
            host_port,
            players: Vec::with_capacity(capacity),
            host_connection: None,
            last_requests: HashMap::new(),
            closed: false,
            revision: Revision(0),
            next_avatar_id: 1,
            seat_count,
        }
    }

    pub(super) fn player_id(&self, connection: ConnectionId) -> Option<PlayerId> {
        self.players
            .iter()
            .find(|player| player.connection == connection && !player.left)
            .map(|player| player.id)
    }

    pub(super) fn record_received_interaction(
        &mut self,
        target: PlayerId,
        kind: PlayerInteractionKind,
    ) {
        let Some(participant) = self
            .players
            .iter_mut()
            .find(|participant| participant.id == target && !participant.left)
        else {
            return;
        };
        let stats = participant
            .game_profiles
            .interactions
            .get_or_insert_with(PlayerInteractionStats::default);
        match kind {
            PlayerInteractionKind::Flower => {
                stats.flowers_received = stats.flowers_received.saturating_add(1);
            }
            PlayerInteractionKind::Wine => {
                stats.flowers_received = stats.flowers_received.saturating_add(10);
            }
            PlayerInteractionKind::Egg => {
                stats.eggs_received = stats.eggs_received.saturating_add(1);
            }
            PlayerInteractionKind::Shoe => {
                stats.eggs_received = stats.eggs_received.saturating_add(10);
            }
        }
    }

    pub(super) fn begin_request(
        &mut self,
        connection: ConnectionId,
        message: &ClientMessage,
    ) -> Result<(), Vec<Delivery>> {
        if message.protocol_version != PROTOCOL_VERSION {
            return Err(self.reject(
                connection,
                message.request_id,
                RejectReason::Request(RequestViolation::ProtocolMismatch {
                    expected: PROTOCOL_VERSION,
                    received: message.protocol_version,
                }),
            ));
        }
        if message.room_id != self.room_id {
            return Err(self.reject(
                connection,
                message.request_id,
                RejectReason::Request(RequestViolation::RoomMismatch),
            ));
        }
        if let Some(last_seen) = self.last_requests.get(&connection).copied() {
            if message.request_id == last_seen {
                return Err(self.reject(
                    connection,
                    message.request_id,
                    RejectReason::Request(RequestViolation::DuplicateRequest { last_seen }),
                ));
            }
            if message.request_id < last_seen {
                return Err(self.reject(
                    connection,
                    message.request_id,
                    RejectReason::Request(RequestViolation::StaleRequest { last_seen }),
                ));
            }
        }
        self.last_requests.insert(connection, message.request_id);
        Ok(())
    }

    pub(super) fn join(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        request: JoinRequest,
        game_started: bool,
        capacity: u8,
    ) -> Result<Vec<Delivery>, RejectReason> {
        if self.player_id(connection).is_some() {
            return Err(RejectReason::Player(PlayerViolation::AlreadyJoined));
        }
        let normalized_name = request.name.trim();
        if normalized_name.is_empty() {
            return Err(RejectReason::Player(PlayerViolation::NameEmpty));
        }
        if normalized_name.chars().count() > MAX_PLAYER_NAME_CHARS {
            return Err(RejectReason::Player(PlayerViolation::NameTooLong {
                max_chars: MAX_PLAYER_NAME_CHARS as u16,
            }));
        }
        if !crate::valid_identity_proof(self.room_id, &request) {
            return Err(RejectReason::Player(PlayerViolation::InvalidIdentityProof));
        }
        let JoinRequest {
            name,
            reconnect_token,
            profile_id,
            reference_points,
            completed_games,
            game_profiles,
            identity_signature: _,
        } = request;
        let name = name.trim();
        if let Some(index) = self
            .players
            .iter()
            .position(|player| player.reconnect_token == reconnect_token && !player.left)
        {
            if self.players[index].name != name || self.players[index].profile_id != profile_id {
                return Err(RejectReason::Player(PlayerViolation::AlreadyJoined));
            }
            return Ok(self.reconnect(index, connection, request_id, game_started));
        }
        if self
            .players
            .iter()
            .any(|player| player.profile_id == profile_id && !player.left)
        {
            return Err(RejectReason::Player(PlayerViolation::AlreadyJoined));
        }
        if game_started {
            return Err(RejectReason::Game(GameViolation::GameAlreadyStarted));
        }
        if self.players.iter().filter(|player| !player.left).count() >= usize::from(capacity) {
            return Err(RejectReason::Room(RoomViolation::RoomFull));
        }

        let vacant = self.players.iter().position(|player| player.left);
        let player = PlayerId(
            vacant
                .unwrap_or(self.players.len())
                .try_into()
                .expect("a room contains at most six active player slots"),
        );
        let joining_as_host = self.host_connection.is_none();
        if joining_as_host {
            self.host_connection = Some(connection);
        }
        let seat = self
            .random_available_seat()
            .expect("a non-full room always has an available seat");
        let participant = Participant {
            id: player,
            profile_id,
            connection,
            name: name.to_owned(),
            avatar: None,
            avatar_png: None,
            reconnect_token,
            seat: Some(seat),
            ready: joining_as_host,
            connected: true,
            auto_play: false,
            is_bot: false,
            left: false,
            reference_points,
            completed_games,
            game_profiles,
        };
        if let Some(index) = vacant {
            self.players[index] = participant;
        } else {
            self.players.push(participant);
        }
        self.bump_revision();
        Ok(self.join_deliveries(connection, request_id, player))
    }

    fn reconnect(
        &mut self,
        index: usize,
        connection: ConnectionId,
        request_id: RequestId,
        game_started: bool,
    ) -> Vec<Delivery> {
        let previous_connection = self.players[index].connection;
        let player = self.players[index].id;
        let reassigned_seat = (!game_started && self.players[index].seat.is_none())
            .then(|| self.random_available_seat())
            .flatten();
        self.players[index].connection = connection;
        self.players[index].connected = true;
        if let Some(seat) = reassigned_seat {
            self.players[index].seat = Some(seat);
        }
        if self.host_connection == Some(previous_connection) {
            self.host_connection = Some(connection);
        }
        let previous_last_request = self.last_requests.remove(&previous_connection);
        if let Some(last_request) = self.last_requests.get_mut(&connection) {
            if let Some(previous_last_request) = previous_last_request {
                *last_request = (*last_request).max(previous_last_request);
            }
        } else if let Some(previous_last_request) = previous_last_request {
            self.last_requests.insert(connection, previous_last_request);
        }
        self.bump_revision();
        self.join_deliveries(connection, request_id, player)
    }

    fn join_deliveries(
        &self,
        connection: ConnectionId,
        request_id: RequestId,
        player: PlayerId,
    ) -> Vec<Delivery> {
        let mut deliveries = vec![self.delivery(
            connection,
            Some(request_id),
            ServerEvent::Joined { you: player },
        )];
        deliveries.extend(
            self.players
                .iter()
                .filter(|player| !player.left)
                .filter_map(|participant| {
                    Some(self.delivery(
                        connection,
                        None,
                        ServerEvent::AvatarData {
                            id: participant.avatar?,
                            png: participant.avatar_png.clone()?,
                        },
                    ))
                }),
        );
        deliveries
    }

    pub(super) fn set_avatar(
        &mut self,
        connection: ConnectionId,
        png: Vec<u8>,
        game_started: bool,
    ) -> Result<Vec<Delivery>, RejectReason> {
        let player = self
            .player_id(connection)
            .ok_or(RejectReason::Player(PlayerViolation::NotJoined))?;
        if game_started {
            return Err(RejectReason::Game(GameViolation::GameAlreadyStarted));
        }
        if self.players[usize::from(player.0)].avatar.is_some() {
            return Err(RejectReason::Player(PlayerViolation::AvatarAlreadySet));
        }
        if !crate::valid_avatar_png(&png) {
            return Err(RejectReason::Player(PlayerViolation::InvalidAvatar));
        }
        let avatar = AvatarId(self.next_avatar_id);
        self.next_avatar_id += 1;
        let participant = &mut self.players[usize::from(player.0)];
        participant.avatar = Some(avatar);
        participant.avatar_png = Some(png.clone());
        self.bump_revision();
        Ok(self.broadcast_event(None, ServerEvent::AvatarData { id: avatar, png }))
    }

    pub(super) fn select_seat(
        &mut self,
        connection: ConnectionId,
        seat: SeatId,
        game_started: bool,
    ) -> Result<(), RejectReason> {
        let player = self
            .player_id(connection)
            .ok_or(RejectReason::Player(PlayerViolation::NotJoined))?;
        if game_started {
            return Err(RejectReason::Game(GameViolation::GameAlreadyStarted));
        }
        if seat.0 >= self.seat_count {
            return Err(RejectReason::Room(RoomViolation::InvalidSeat));
        }
        if self.players.iter().any(|participant| {
            !participant.left && participant.id != player && participant.seat == Some(seat)
        }) {
            return Err(RejectReason::Room(RoomViolation::SeatTaken));
        }
        let is_host = self.host_connection == Some(connection);
        let participant = &mut self.players[usize::from(player.0)];
        if participant.seat != Some(seat) {
            participant.seat = Some(seat);
            participant.ready = is_host;
            self.bump_revision();
        }
        Ok(())
    }

    pub(super) fn set_ready(
        &mut self,
        connection: ConnectionId,
        ready: bool,
        game_started: bool,
    ) -> Result<(), RejectReason> {
        let player = self
            .player_id(connection)
            .ok_or(RejectReason::Player(PlayerViolation::NotJoined))?;
        if game_started {
            return Err(RejectReason::Game(GameViolation::GameAlreadyStarted));
        }
        let ready = if self.host_connection == Some(connection) {
            true
        } else {
            ready
        };
        if ready && self.players[usize::from(player.0)].seat.is_none() {
            return Err(RejectReason::Room(RoomViolation::MustSelectSeat));
        }
        if self.players[usize::from(player.0)].ready != ready {
            self.players[usize::from(player.0)].ready = ready;
            self.bump_revision();
        }
        Ok(())
    }

    pub(super) fn configure_bot_seat(
        &mut self,
        connection: ConnectionId,
        seat: SeatId,
        occupied: bool,
        game_started: bool,
    ) -> Result<(), RejectReason> {
        #[cfg(not(feature = "developer"))]
        {
            let _ = (connection, seat, occupied, game_started);
            Err(RejectReason::Game(
                GameViolation::DeveloperFeatureUnavailable,
            ))
        }
        #[cfg(feature = "developer")]
        {
            self.player_id(connection)
                .ok_or(RejectReason::Player(PlayerViolation::NotJoined))?;
            if self.host_connection != Some(connection) {
                return Err(RejectReason::Room(RoomViolation::OnlyHostCanConfigure));
            }
            if game_started {
                return Err(RejectReason::Game(GameViolation::GameAlreadyStarted));
            }
            if seat.0 >= self.seat_count {
                return Err(RejectReason::Room(RoomViolation::InvalidSeat));
            }
            let occupant = self
                .players
                .iter()
                .position(|player| !player.left && player.seat == Some(seat));
            if let Some(index) = occupant {
                if !self.players[index].is_bot {
                    return Err(RejectReason::Room(RoomViolation::SeatTaken));
                }
                if occupied {
                    return Ok(());
                }
                let player = &mut self.players[index];
                player.seat = None;
                player.ready = false;
                player.connected = false;
                player.auto_play = false;
                player.left = true;
                self.bump_revision();
                return Ok(());
            }
            if !occupied {
                return Ok(());
            }

            let ordinal = (1..=self.seat_count)
                .find(|ordinal| {
                    let name = format!("机器人{ordinal}");
                    self.players
                        .iter()
                        .all(|player| player.left || player.name != name)
                })
                .expect("there are at most six robot seats");
            let vacant = self.players.iter().position(|player| player.left);
            let id = PlayerId(vacant.unwrap_or(self.players.len()) as u8);
            let participant = Participant {
                id,
                profile_id: ProfileId([0; 32]),
                connection: ConnectionId(u64::MAX - u64::from(seat.0)),
                name: format!("机器人{ordinal}"),
                avatar: None,
                avatar_png: None,
                reconnect_token: ReconnectToken(u64::MAX - u64::from(seat.0)),
                seat: Some(seat),
                ready: true,
                connected: false,
                auto_play: true,
                is_bot: true,
                left: false,
                reference_points: 0,
                completed_games: 0,
                game_profiles: PlayerGameProfiles::default(),
            };
            if let Some(index) = vacant {
                self.players[index] = participant;
            } else {
                self.players.push(participant);
            }
            self.bump_revision();
            Ok(())
        }
    }

    pub(super) fn chat(
        &self,
        connection: ConnectionId,
        request_id: RequestId,
        content: ChatContent,
        game_started: bool,
    ) -> Result<Vec<Delivery>, RejectReason> {
        let source = self
            .player_id(connection)
            .ok_or(RejectReason::Player(PlayerViolation::NotJoined))?;
        if !game_started {
            return Err(RejectReason::Game(GameViolation::GameNotStarted));
        }
        let content = match content {
            ChatContent::Text(text) => {
                let text = text.trim();
                if text.is_empty() || text.chars().count() > MAX_CHAT_MESSAGE_CHARS {
                    return Err(RejectReason::Room(RoomViolation::InvalidChatMessage));
                }
                ChatContent::Text(text.to_owned())
            }
            ChatContent::QuickVoice(index) if index < QUICK_VOICE_COUNT => {
                ChatContent::QuickVoice(index)
            }
            ChatContent::QuickVoice(_) => {
                return Err(RejectReason::Room(RoomViolation::InvalidChatMessage));
            }
            ChatContent::Emoji(emoji) => ChatContent::Emoji(emoji),
        };
        let chat = ChatMessage { source, content };
        Ok(self.broadcast_event(
            Some((connection, request_id)),
            ServerEvent::ChatMessage(chat),
        ))
    }

    pub(super) fn close_room(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
    ) -> Result<Vec<Delivery>, RejectReason> {
        self.player_id(connection)
            .ok_or(RejectReason::Player(PlayerViolation::NotJoined))?;
        if self.host_connection != Some(connection) {
            return Err(RejectReason::Room(RoomViolation::OnlyHostCanCloseRoom));
        }
        self.bump_revision();
        let deliveries =
            self.broadcast_event(Some((connection, request_id)), ServerEvent::RoomClosed);
        self.closed = true;
        Ok(deliveries)
    }

    pub(super) fn host_player_id(&self) -> Option<PlayerId> {
        self.host_connection.and_then(|connection| {
            self.players
                .iter()
                .find(|player| player.connection == connection && !player.left)
                .map(|player| player.id)
        })
    }

    pub(super) fn bump_revision(&mut self) {
        self.revision.0 += 1;
    }

    pub(super) fn reject(
        &self,
        recipient: ConnectionId,
        request_id: RequestId,
        reason: RejectReason,
    ) -> Vec<Delivery> {
        vec![self.delivery(
            recipient,
            Some(request_id),
            ServerEvent::Rejected { reason },
        )]
    }

    pub(super) fn delivery(
        &self,
        recipient: ConnectionId,
        in_reply_to: Option<RequestId>,
        event: ServerEvent,
    ) -> Delivery {
        Delivery {
            recipient,
            message: ServerMessage {
                protocol_version: PROTOCOL_VERSION,
                room_id: self.room_id,
                revision: self.revision,
                in_reply_to,
                event,
            },
        }
    }

    pub(super) fn lobby_snapshot(&self, game: GameKind, rules: GameRules) -> LobbySnapshot {
        LobbySnapshot {
            game,
            rules,
            host_port: self.host_port,
            host: self.host_player_id(),
            players: self
                .players
                .iter()
                .filter(|player| !player.left)
                .map(|player| player.public_metadata().into())
                .collect(),
        }
    }

    pub(super) fn broadcast_lobby(
        &self,
        game: GameKind,
        rules: GameRules,
        origin: Option<(ConnectionId, RequestId)>,
    ) -> Vec<Delivery> {
        let snapshot = self.lobby_snapshot(game, rules);
        self.broadcast_event(origin, ServerEvent::LobbySnapshot(snapshot))
    }

    pub(super) fn broadcast_event(
        &self,
        origin: Option<(ConnectionId, RequestId)>,
        event: ServerEvent,
    ) -> Vec<Delivery> {
        self.broadcast_private(origin, |_| event.clone())
    }

    pub(super) fn broadcast_private(
        &self,
        origin: Option<(ConnectionId, RequestId)>,
        mut event: impl FnMut(&Participant) -> ServerEvent,
    ) -> Vec<Delivery> {
        self.active_participants()
            .map(|player| {
                let reply = origin
                    .filter(|(connection, _)| *connection == player.connection)
                    .map(|(_, request)| request);
                self.delivery(player.connection, reply, event(player))
            })
            .collect()
    }

    pub(super) fn broadcast_game_events<E>(
        &self,
        events: impl IntoIterator<Item = E>,
    ) -> Vec<Delivery>
    where
        E: Into<GameEvent>,
    {
        events
            .into_iter()
            .flat_map(|event| self.broadcast_event(None, ServerEvent::GameEvent(event.into())))
            .collect()
    }

    fn random_available_seat(&self) -> Option<SeatId> {
        let mut available = (0..self.seat_count)
            .map(SeatId)
            .filter(|seat| {
                !self
                    .players
                    .iter()
                    .any(|player| !player.left && player.seat == Some(*seat))
            })
            .collect::<Vec<_>>();
        fastrand::shuffle(&mut available);
        available.pop()
    }

    pub(super) fn remove_departed_players(&mut self) {
        self.players.retain(|player| !player.left);
        self.players
            .sort_by_key(|player| player.seat.map_or(u8::MAX, |seat| seat.0));
        for (index, player) in self.players.iter_mut().enumerate() {
            player.id = PlayerId(index as u8);
        }
    }

    /// 进入终局准备阶段：仍在托管的玩家与机器人立即准备下一局，并保留托管状态。
    pub(super) fn prepare_rematch(&mut self) {
        for player in &mut self.players {
            player.ready = player.is_bot || player.auto_play;
        }
    }

    pub(super) fn reset_ready_after_rules_change(&mut self) {
        let host = self.host_connection;
        for player in &mut self.players {
            player.ready = player.is_bot || host == Some(player.connection);
        }
    }

    #[cfg(feature = "developer")]
    pub(super) fn remove_developer_bots(&mut self) {
        for player in self.players.iter_mut().filter(|player| player.is_bot) {
            player.seat = None;
            player.ready = false;
            player.connected = false;
            player.auto_play = false;
            player.left = true;
        }
    }
}
