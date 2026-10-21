use super::RoomActivity;
use crate::achievements::RoomAchievements;
use crate::{ConnectionId, Delivery};
use leocard_protocol::{
    AvatarId, ChatContent, ChatMessage, ClientMessage, GameEvent, GameEventContext, GameKind,
    GameRules, GameViolation, LobbySnapshot, MAX_CHAT_MESSAGE_CHARS, MatchId, PROTOCOL_VERSION,
    PlayerGameProfiles, PlayerId, PlayerInteraction, PlayerInteractionKind, PlayerInteractionStats,
    PlayerViolation, ProfileId, QUICK_VOICE_COUNT, ReconnectToken, RejectReason, RequestId,
    RequestViolation, Revision, RoomId, RoomViolation, SeatId, ServerEvent, ServerMessage,
    TABLE_SEAT_COUNT,
};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub(crate) struct Participant {
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
    /// 房主使用此字段执行游戏对应的托管策略。
    pub auto_play: bool,
    /// 房主添加的托管座位；机器人没有真实网络连接。
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
    pub(crate) achievements: RoomAchievements,
    activity: RoomActivity,
    pub(crate) room_id: RoomId,
    pub(crate) host_port: u16,
    pub(crate) players: Vec<Participant>,
    pub(super) spectators: Vec<Participant>,
    pub(crate) host_connection: Option<ConnectionId>,
    pub(crate) last_requests: HashMap<ConnectionId, RequestId>,
    pub(crate) closed: bool,
    pub(crate) revision: Revision,
    pub(crate) next_avatar_id: u64,
    pub(crate) seat_count: u8,
}

impl RoomSession {
    pub(super) fn active_participants(&self) -> impl Iterator<Item = &Participant> {
        self.participants()
            .filter(|player| player.connected && !player.left)
    }

    pub(crate) fn heartbeat(&self) -> Vec<Delivery> {
        self.broadcast_event(None, ServerEvent::Heartbeat)
    }

    pub(crate) fn new(room_id: RoomId, host_port: u16, capacity: usize) -> Self {
        Self::new_with_seat_count(room_id, host_port, capacity, TABLE_SEAT_COUNT)
    }

    pub(crate) fn new_with_seat_count(
        room_id: RoomId,
        host_port: u16,
        capacity: usize,
        seat_count: u8,
    ) -> Self {
        assert!(seat_count > 0 && capacity <= usize::from(seat_count));
        Self {
            achievements: RoomAchievements::default(),
            activity: RoomActivity::default(),
            room_id,
            host_port,
            players: Vec::with_capacity(capacity),
            spectators: Vec::new(),
            host_connection: None,
            last_requests: HashMap::new(),
            closed: false,
            revision: Revision(0),
            next_avatar_id: 1,
            seat_count,
        }
    }

    pub(crate) fn player_id(&self, connection: ConnectionId) -> Option<PlayerId> {
        self.participants()
            .find(|player| player.connection == connection && !player.left)
            .map(|player| player.id)
    }

    pub(crate) fn record_interaction(
        &mut self,
        source: PlayerId,
        target: PlayerId,
        kind: PlayerInteractionKind,
        seed: u32,
    ) -> PlayerInteraction {
        self.record_received_interaction(target, kind);
        self.activity.interaction(source, target, kind, seed)
    }

    fn record_received_interaction(&mut self, target: PlayerId, kind: PlayerInteractionKind) {
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
        stats.flowers_received = stats.flowers_received.saturating_add(kind.flower_count());
        stats.eggs_received = stats.eggs_received.saturating_add(kind.egg_count());
    }

    pub(crate) fn begin_request(
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

    pub(crate) fn set_avatar(
        &mut self,
        connection: ConnectionId,
        png: Vec<u8>,
    ) -> Result<Vec<Delivery>, RejectReason> {
        let player = self
            .player_id(connection)
            .ok_or(RejectReason::Player(PlayerViolation::NotJoined))?;
        let participant = self.participant_mut(player).expect("joined participant");
        if participant.avatar.is_some() {
            if participant.avatar_png.as_deref() == Some(png.as_slice()) {
                return Ok(Vec::new());
            }
            return Err(RejectReason::Player(PlayerViolation::AvatarAlreadySet));
        }
        if !crate::valid_avatar_png(&png) {
            return Err(RejectReason::Player(PlayerViolation::InvalidAvatar));
        }
        let avatar = AvatarId(self.next_avatar_id);
        self.next_avatar_id += 1;
        let participant = self.participant_mut(player).expect("joined participant");
        participant.avatar = Some(avatar);
        participant.avatar_png = Some(png.clone());
        self.bump_revision();
        Ok(self.broadcast_event(None, ServerEvent::AvatarData { id: avatar, png }))
    }

    pub(crate) fn select_seat(
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

    pub(crate) fn set_ready(
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

    pub(crate) fn chat(
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

    pub(crate) fn close_room(
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

    pub(crate) fn host_player_id(&self) -> Option<PlayerId> {
        self.host_connection.and_then(|connection| {
            self.players
                .iter()
                .find(|player| player.connection == connection && !player.left)
                .map(|player| player.id)
        })
    }

    pub(crate) fn bump_revision(&mut self) {
        self.revision.0 += 1;
    }

    pub(crate) fn reject(
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

    pub(crate) fn delivery(
        &self,
        recipient: ConnectionId,
        in_reply_to: Option<RequestId>,
        event: ServerEvent,
    ) -> Delivery {
        Delivery {
            recipient,
            message: ServerMessage {
                game_context: None,
                protocol_version: PROTOCOL_VERSION,
                room_id: self.room_id,
                revision: self.revision,
                in_reply_to,
                event,
            },
        }
    }

    pub(crate) fn lobby_snapshot(&self, game: GameKind, rules: GameRules) -> LobbySnapshot {
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

    pub(crate) fn broadcast_lobby(
        &self,
        game: GameKind,
        rules: GameRules,
        origin: Option<(ConnectionId, RequestId)>,
    ) -> Vec<Delivery> {
        let snapshot = self.lobby_snapshot(game, rules);
        self.broadcast_event(origin, ServerEvent::LobbySnapshot(snapshot))
    }

    pub(crate) fn broadcast_event(
        &self,
        origin: Option<(ConnectionId, RequestId)>,
        event: ServerEvent,
    ) -> Vec<Delivery> {
        self.broadcast_private(origin, |_| event.clone())
    }

    pub(crate) fn broadcast_private(
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

    pub(crate) fn broadcast_game_events<E>(
        &self,
        events: impl IntoIterator<Item = E>,
        origin: Option<(MatchId, Option<u32>)>,
    ) -> Vec<Delivery>
    where
        E: Into<GameEvent>,
    {
        events
            .into_iter()
            .enumerate()
            .flat_map(|(index, event)| {
                let context = origin.map(|(match_id, hand_index)| GameEventContext {
                    records_progress: self.records_game_progress(),
                    match_id,
                    hand_index,
                    sequence: (u128::from(self.revision.0) << 64) | index as u128,
                });
                let mut deliveries =
                    self.broadcast_event(None, ServerEvent::GameEvent(event.into()));
                for delivery in &mut deliveries {
                    delivery.message.game_context = context;
                }
                deliveries
            })
            .collect()
    }

    pub(super) fn random_available_seat(&self) -> Option<SeatId> {
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

    pub(crate) fn remove_departed_players(&mut self) {
        self.players.retain(|player| !player.left);
        self.players
            .sort_by_key(|player| player.seat.map_or(u8::MAX, |seat| seat.0));
        for (index, player) in self.players.iter_mut().enumerate() {
            player.id = PlayerId(index as u8);
        }
    }

    /// 进入终局准备阶段：仍在托管的玩家与机器人立即准备下一局，并保留托管状态。
    pub(crate) fn prepare_rematch(&mut self) {
        for player in &mut self.players {
            player.ready = player.is_bot || player.auto_play;
        }
    }

    pub(crate) fn reset_ready_after_rules_change(&mut self) {
        let host = self.host_connection;
        for player in &mut self.players {
            player.ready = player.is_bot || host == Some(player.connection);
        }
    }
}
