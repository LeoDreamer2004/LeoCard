use std::collections::HashSet;
use std::ops::{Deref, DerefMut};
use std::time::Duration;

use leocard_protocol::{
    ChatContent, ClientCommand, ClientMessage, GameCommand, GameEvent, GameKind, GamePhaseView,
    GameRules, GameSnapshot, GameViolation, LobbySnapshot, MAX_PLAYER_NAME_CHARS, MatchId,
    PlayerGameProfiles, PlayerId, PlayerInteraction, PlayerInteractionKind, PlayerPublicState,
    PlayerReferenceChange, PlayerScore, ProfileId, PublicPlay, PublicPlayRecord, QiGui523Command,
    QiGui523Event, QiGui523ProfileStats, QiGui523Snapshot, ReconnectToken, RejectReason, RequestId,
    RevealedHand, Revision, RoomId, RuleViolation, SeatId, ServerEvent, StartingCardView,
    TABLE_SEAT_COUNT, TrickView, TurnTimerView,
};
use leocard_qigui523::{
    Card, GameError, GameState, Phase, PlayError, PlayKind, PlayRecord, PlayerId as CorePlayerId,
    QiGui523Bot, QiGui523BotRequest, RuleSet, build_deck, classify, reference_point_deltas,
};

use crate::room::Participant;
use crate::{
    AUTO_PLAY_DELAY, AutoPlayDelayState, ConnectionId, Delivery, HostError, RoomSession,
    TurnTimerState, new_match_id, valid_identity_proof,
};

/// 单房间权威会话。所有命令均按调用顺序串行处理。
#[derive(Clone, Debug)]
pub struct QiGui523Session {
    room: RoomSession,
    rules: RuleSet,
    shuffled_deck: Option<Vec<Card>>,
    game: Option<GameState>,
    match_id: Option<MatchId>,
    finished_reference_changes: Option<Vec<PlayerReferenceChange>>,
    match_profile_stats: Vec<QiGui523ProfileStats>,
    turn_timer: Option<TurnTimerState>,
    auto_play_delay: Option<AutoPlayDelayState>,
}

impl Deref for QiGui523Session {
    type Target = RoomSession;

    fn deref(&self) -> &Self::Target {
        &self.room
    }
}

impl DerefMut for QiGui523Session {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.room
    }
}

impl QiGui523Session {
    /// `shuffled_deck[0]` 是第一张发出的牌；房主应在创建会话前完成洗牌。
    pub fn new(
        room_id: RoomId,
        rules: RuleSet,
        shuffled_deck: Vec<Card>,
    ) -> Result<Self, HostError> {
        Self::new_with_host_port(room_id, 52300, rules, shuffled_deck)
    }

    pub fn new_with_host_port(
        room_id: RoomId,
        host_port: u16,
        rules: RuleSet,
        shuffled_deck: Vec<Card>,
    ) -> Result<Self, HostError> {
        let rules = rules.validate()?;
        validate_deck(rules.deck_count, &shuffled_deck)?;
        Ok(Self {
            room: RoomSession::new(room_id, host_port, usize::from(rules.player_count)),
            rules,
            shuffled_deck: Some(shuffled_deck),
            game: None,
            match_id: None,
            finished_reference_changes: None,
            match_profile_stats: Vec::new(),
            turn_timer: None,
            auto_play_delay: None,
        })
    }

    pub fn room_id(&self) -> RoomId {
        self.room_id
    }

    pub fn rules(&self) -> &RuleSet {
        &self.rules
    }

    pub fn revision(&self) -> Revision {
        self.revision
    }

    pub fn game(&self) -> Option<&GameState> {
        self.game.as_ref()
    }

    pub fn is_closed(&self) -> bool {
        self.closed
    }

    /// 生成不改变权威修订号的轻量心跳，只发给当前在线玩家。
    pub fn heartbeat(&self) -> Vec<Delivery> {
        self.players
            .iter()
            .filter(|player| player.connected)
            .map(|player| self.delivery(player.connection, None, ServerEvent::Heartbeat))
            .collect()
    }

    /// 推进权威出牌计时。TCP 层传入真实经过时间；测试和其他宿主也可确定性调用。
    pub fn advance_time(&mut self, elapsed: Duration) -> Vec<Delivery> {
        if elapsed.is_zero() || self.game.is_none() {
            return Vec::new();
        }
        if self.current_player_is_disconnected() {
            self.auto_play_delay = None;
            let effect = self.play_automatic_action();
            self.reset_timer_for_current_turn();
            self.bump_revision();
            return self.broadcast_game_after_action(None, effect);
        }
        if let Some(player) = self.current_auto_play_player() {
            let delay = self.auto_play_delay.get_or_insert(AutoPlayDelayState {
                player,
                remaining: AUTO_PLAY_DELAY,
            });
            if delay.player != player {
                *delay = AutoPlayDelayState {
                    player,
                    remaining: AUTO_PLAY_DELAY,
                };
            }
            if elapsed < delay.remaining {
                delay.remaining -= elapsed;
                return Vec::new();
            }
            self.auto_play_delay = None;
            let effect = self.play_automatic_action();
            self.reset_timer_for_current_turn();
            self.bump_revision();
            return self.broadcast_game_after_action(None, effect);
        }
        self.auto_play_delay = None;
        if self.turn_timer.is_none() {
            return Vec::new();
        }

        let before = self.turn_timer_view();
        let expired = {
            let timer = self.turn_timer.as_mut().expect("checked above");
            let player_index = usize::from(timer.player.0);
            let base_used = elapsed.min(timer.base_remaining);
            timer.base_remaining -= base_used;
            let reserve_used = (elapsed - base_used).min(timer.reserve_remaining[player_index]);
            timer.reserve_remaining[player_index] -= reserve_used;
            timer.base_remaining.is_zero() && timer.reserve_remaining[player_index].is_zero()
        };

        if expired {
            let effect = self.play_automatic_action();
            self.reset_timer_for_current_turn();
            self.bump_revision();
            return self.broadcast_game_after_action(None, effect);
        }

        if self.turn_timer_view() != before {
            self.bump_revision();
            self.broadcast_game_after_update(None)
        } else {
            Vec::new()
        }
    }

    /// 标记连接离线并向仍在线的玩家广播新快照。
    pub fn disconnect(&mut self, connection: ConnectionId) -> Vec<Delivery> {
        let Some(index) = self
            .players
            .iter()
            .position(|player| player.connection == connection && player.connected)
        else {
            return Vec::new();
        };
        self.players[index].connected = false;
        if self.host_connection == Some(connection) {
            self.bump_revision();
            self.closed = true;
            return self
                .players
                .iter()
                .filter(|player| player.connected && !player.left)
                .map(|player| self.delivery(player.connection, None, ServerEvent::RoomClosed))
                .collect();
        }
        if self.game.is_none() {
            self.players[index].seat = None;
            self.players[index].ready = false;
            // 大厅没有需要恢复的私有牌局状态。若仍把断线玩家保留为活跃参与者，
            // 快照人数和房间容量都会被一个不可见的“幽灵玩家”占用。自动重连仍可
            // 使用同一身份重新加入，并复用这个已离开的槽位。
            self.players[index].left = true;
        }
        let effect = if self.current_player_is_disconnected() {
            let effect = self.play_automatic_action();
            self.reset_timer_for_current_turn();
            effect
        } else {
            None
        };
        self.bump_revision();
        if self.game.is_some() {
            self.broadcast_game_after_action(None, effect)
        } else {
            self.broadcast_lobby(None)
        }
    }

    pub fn handle(&mut self, connection: ConnectionId, message: ClientMessage) -> Vec<Delivery> {
        if let Err(deliveries) = self.room.begin_request(connection, &message) {
            return deliveries;
        }
        // A finished match may be observed again through ready/lobby/snapshot commands.
        // Applying here keeps the settlement idempotent even outside the normal final-play path.
        self.apply_finished_reference_points();

        match message.command {
            ClientCommand::Join {
                name,
                reconnect_token,
                profile_id,
                reference_points,
                completed_games,
                game_profiles,
                identity_signature,
            } => self.join(
                connection,
                message.request_id,
                name,
                reconnect_token,
                profile_id,
                reference_points,
                completed_games,
                game_profiles,
                identity_signature,
            ),
            ClientCommand::SetAvatar { png } => {
                self.set_avatar(connection, message.request_id, png)
            }
            ClientCommand::SelectSeat { seat } => {
                self.select_seat(connection, message.request_id, seat)
            }
            ClientCommand::ConfigureBotSeat { seat, occupied } => {
                match self
                    .room
                    .configure_bot_seat(connection, seat, occupied, self.game.is_some())
                {
                    Ok(()) => self.broadcast_lobby(Some((connection, message.request_id))),
                    Err(reason) => self.reject(connection, message.request_id, reason),
                }
            }
            ClientCommand::SetReady { ready } => {
                self.set_ready(connection, message.request_id, ready)
            }
            ClientCommand::Game(GameCommand::QiGui523(command)) => match command {
                QiGui523Command::SetAutoPlay { enabled } => {
                    self.set_auto_play(connection, message.request_id, enabled)
                }
                QiGui523Command::UpdateRules { rules } => {
                    self.update_rules(connection, message.request_id, rules)
                }
                QiGui523Command::PlayCards { cards } => {
                    self.play_cards(connection, message.request_id, &cards)
                }
                QiGui523Command::SetDeveloperHand { cards } => {
                    self.set_developer_hand(connection, message.request_id, cards)
                }
                QiGui523Command::Pass => self.pass(connection, message.request_id),
            },
            ClientCommand::Game(GameCommand::TexasHoldem(_)) => self.reject(
                connection,
                message.request_id,
                RejectReason::WrongGame {
                    expected: GameKind::QiGui523,
                    received: GameKind::TexasHoldem,
                },
            ),
            ClientCommand::Game(GameCommand::Shengji(_)) => self.reject(
                connection,
                message.request_id,
                RejectReason::WrongGame {
                    expected: GameKind::QiGui523,
                    received: GameKind::Shengji,
                },
            ),
            ClientCommand::Game(GameCommand::Uno(_)) => self.reject(
                connection,
                message.request_id,
                RejectReason::WrongGame {
                    expected: GameKind::QiGui523,
                    received: GameKind::Uno,
                },
            ),
            ClientCommand::StartGame => self.start_game(connection, message.request_id),
            ClientCommand::ReturnToLobby => self.return_to_lobby(connection, message.request_id),
            ClientCommand::PlayAgain => self.play_again(connection, message.request_id),
            ClientCommand::LeaveRoom => self.leave_room(connection, message.request_id),
            ClientCommand::CloseRoom => self.close_room(connection, message.request_id),
            ClientCommand::Interact { target, kind } => {
                self.interact(connection, message.request_id, target, kind)
            }
            ClientCommand::Chat { content } => self.chat(connection, message.request_id, content),
            ClientCommand::RequestSnapshot => self.snapshot(connection, message.request_id),
        }
    }

    fn chat(
        &self,
        connection: ConnectionId,
        request_id: RequestId,
        content: ChatContent,
    ) -> Vec<Delivery> {
        self.room
            .chat(connection, request_id, content, self.game.is_some())
            .unwrap_or_else(|reason| self.reject(connection, request_id, reason))
    }

    #[cfg(feature = "developer")]
    fn set_developer_hand(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        cards: Vec<Card>,
    ) -> Vec<Delivery> {
        let Some(player) = self.player_id(connection) else {
            return self.reject(connection, request_id, RejectReason::NotJoined);
        };
        let Some(game) = self.game.as_mut() else {
            return self.reject(connection, request_id, RejectReason::GameNotStarted);
        };
        if cards.is_empty()
            || game
                .replace_player_hand(to_core_player(player), cards)
                .is_err()
        {
            return self.reject(connection, request_id, RejectReason::InvalidDeveloperHand);
        }
        self.bump_revision();
        self.broadcast_game(Some((connection, request_id)))
    }

    #[cfg(not(feature = "developer"))]
    fn set_developer_hand(
        &self,
        connection: ConnectionId,
        request_id: RequestId,
        _cards: Vec<Card>,
    ) -> Vec<Delivery> {
        self.reject(
            connection,
            request_id,
            RejectReason::DeveloperFeatureUnavailable,
        )
    }

    fn interact(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        target: PlayerId,
        kind: PlayerInteractionKind,
    ) -> Vec<Delivery> {
        let Some(source) = self.player_id(connection) else {
            return self.reject(connection, request_id, RejectReason::NotJoined);
        };
        let Some(game) = self.game.as_ref() else {
            return self.reject(connection, request_id, RejectReason::GameNotStarted);
        };
        if !matches!(game.phase(), Phase::Playing) {
            return self.reject(
                connection,
                request_id,
                RejectReason::GameViolation(GameViolation::QiGui523(
                    RuleViolation::GameAlreadyFinished,
                )),
            );
        }
        if source == target
            || !self
                .players
                .iter()
                .any(|player| player.id == target && !player.left)
        {
            return self.reject(
                connection,
                request_id,
                RejectReason::GameViolation(GameViolation::QiGui523(RuleViolation::InvalidPlayer)),
            );
        }

        let interaction = PlayerInteraction {
            source,
            target,
            kind,
            seed: fastrand::u32(..),
        };
        self.record_received_interaction(target, kind);
        self.bump_revision();
        let mut deliveries = self
            .players
            .iter()
            .filter(|player| player.connected && !player.left)
            .map(|player| {
                self.delivery(
                    player.connection,
                    (player.connection == connection).then_some(request_id),
                    ServerEvent::PlayerInteraction(interaction),
                )
            })
            .collect::<Vec<_>>();
        deliveries.extend(self.broadcast_game(None));
        deliveries
    }

    #[allow(clippy::too_many_arguments)]
    fn join(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        name: String,
        reconnect_token: ReconnectToken,
        profile_id: ProfileId,
        reference_points: i32,
        completed_games: u32,
        game_profiles: PlayerGameProfiles,
        identity_signature: Vec<u8>,
    ) -> Vec<Delivery> {
        if self.player_id(connection).is_some() {
            return self.reject(connection, request_id, RejectReason::AlreadyJoined);
        }
        let name = name.trim();
        if name.is_empty() {
            return self.reject(connection, request_id, RejectReason::NameEmpty);
        }
        if name.chars().count() > MAX_PLAYER_NAME_CHARS {
            return self.reject(
                connection,
                request_id,
                RejectReason::NameTooLong {
                    max_chars: MAX_PLAYER_NAME_CHARS as u16,
                },
            );
        }
        if !valid_identity_proof(
            self.room_id,
            reconnect_token,
            name,
            profile_id,
            reference_points,
            completed_games,
            &game_profiles,
            &identity_signature,
        ) {
            return self.reject(connection, request_id, RejectReason::InvalidIdentityProof);
        }
        if let Some(index) = self
            .players
            .iter()
            .position(|player| player.reconnect_token == reconnect_token && !player.left)
        {
            if self.players[index].name != name || self.players[index].profile_id != profile_id {
                return self.reject(connection, request_id, RejectReason::AlreadyJoined);
            }
            return self.reconnect(index, connection, request_id);
        }
        if self
            .players
            .iter()
            .any(|player| player.profile_id == profile_id && !player.left)
        {
            return self.reject(connection, request_id, RejectReason::AlreadyJoined);
        }
        if self.game.is_some() {
            return self.reject(connection, request_id, RejectReason::GameAlreadyStarted);
        }
        if self.players.iter().filter(|player| !player.left).count()
            >= usize::from(self.rules.player_count)
        {
            return self.reject(connection, request_id, RejectReason::RoomFull);
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
        deliveries.extend(self.broadcast_lobby(Some((connection, request_id))));
        deliveries
    }

    fn reconnect(
        &mut self,
        index: usize,
        connection: ConnectionId,
        request_id: RequestId,
    ) -> Vec<Delivery> {
        let previous_connection = self.players[index].connection;
        let player = self.players[index].id;
        let reassigned_seat = (self.game.is_none() && self.players[index].seat.is_none())
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
        self.last_requests.remove(&previous_connection);
        self.bump_revision();

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
        deliveries.extend(if self.game.is_some() {
            self.broadcast_game(Some((connection, request_id)))
        } else {
            self.broadcast_lobby(Some((connection, request_id)))
        });
        deliveries
    }

    fn set_avatar(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        png: Vec<u8>,
    ) -> Vec<Delivery> {
        match self.room.set_avatar(connection, png, self.game.is_some()) {
            Ok(mut deliveries) => {
                deliveries.extend(self.broadcast_lobby(Some((connection, request_id))));
                deliveries
            }
            Err(reason) => self.reject(connection, request_id, reason),
        }
    }

    fn select_seat(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        seat: SeatId,
    ) -> Vec<Delivery> {
        match self.room.select_seat(connection, seat, self.game.is_some()) {
            Ok(()) => self.broadcast_lobby(Some((connection, request_id))),
            Err(reason) => self.reject(connection, request_id, reason),
        }
    }

    fn set_ready(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        ready: bool,
    ) -> Vec<Delivery> {
        match self.room.set_ready(connection, ready, self.game.is_some()) {
            Ok(()) => self.broadcast_lobby(Some((connection, request_id))),
            Err(reason) => self.reject(connection, request_id, reason),
        }
    }

    fn set_auto_play(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        enabled: bool,
    ) -> Vec<Delivery> {
        let Some(player) = self.player_id(connection) else {
            return self.reject(connection, request_id, RejectReason::NotJoined);
        };
        let Some(game) = self.game.as_ref() else {
            return self.reject(connection, request_id, RejectReason::GameNotStarted);
        };
        if !matches!(game.phase(), Phase::Playing) {
            return self.reject(
                connection,
                request_id,
                RejectReason::GameViolation(GameViolation::QiGui523(
                    RuleViolation::GameAlreadyFinished,
                )),
            );
        }

        let participant = &mut self.players[usize::from(player.0)];
        let changed = participant.auto_play != enabled;
        participant.auto_play = enabled;
        if self.current_player() == Some(player) {
            self.auto_play_delay = enabled.then_some(AutoPlayDelayState {
                player,
                remaining: AUTO_PLAY_DELAY,
            });
        } else if self
            .auto_play_delay
            .as_ref()
            .is_some_and(|delay| delay.player == player)
        {
            self.auto_play_delay = None;
        }
        if changed {
            self.bump_revision();
        }
        self.broadcast_game(Some((connection, request_id)))
    }

    fn update_rules(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        rules: RuleSet,
    ) -> Vec<Delivery> {
        if self.player_id(connection).is_none() {
            return self.reject(connection, request_id, RejectReason::NotJoined);
        }
        if self.game.is_some() {
            return self.reject(connection, request_id, RejectReason::GameAlreadyStarted);
        }
        if self.host_connection != Some(connection) {
            return self.reject(connection, request_id, RejectReason::OnlyHostCanConfigure);
        }
        let rules = RuleSet {
            player_count: TABLE_SEAT_COUNT,
            ..rules
        };
        let Ok(rules) = rules.validate() else {
            return self.reject(
                connection,
                request_id,
                RejectReason::InvalidRuleConfiguration,
            );
        };

        if self.rules != rules {
            self.rules = rules;
            let host_connection = self.host_connection;
            for player in &mut self.players {
                player.ready = host_connection == Some(player.connection);
            }
            let mut deck = build_deck(rules.deck_count);
            fastrand::shuffle(&mut deck);
            self.shuffled_deck = Some(deck);
            self.bump_revision();
        }
        self.broadcast_lobby(Some((connection, request_id)))
    }

    fn start_game(&mut self, connection: ConnectionId, request_id: RequestId) -> Vec<Delivery> {
        if self.player_id(connection).is_none() {
            return self.reject(connection, request_id, RejectReason::NotJoined);
        }
        if self.game.is_some() {
            return self.reject(connection, request_id, RejectReason::GameAlreadyStarted);
        }
        if self.host_connection != Some(connection) {
            return self.reject(connection, request_id, RejectReason::OnlyHostCanStart);
        }
        let active_player_count = self.players.iter().filter(|player| !player.left).count();
        if active_player_count < 2 {
            return self.reject(
                connection,
                request_id,
                RejectReason::NotEnoughPlayers {
                    minimum: 2,
                    actual: active_player_count as u8,
                },
            );
        }
        let not_seated: Vec<_> = self
            .players
            .iter()
            .filter(|player| !player.left)
            .filter(|player| player.seat.is_none())
            .map(|player| player.id)
            .collect();
        if !not_seated.is_empty() {
            return self.reject(connection, request_id, RejectReason::MustSelectSeat);
        }
        let not_ready: Vec<_> = self
            .players
            .iter()
            .filter(|player| !player.left)
            .filter(|player| !player.ready)
            .map(|player| player.id)
            .collect();
        if !not_ready.is_empty() {
            return self.reject(
                connection,
                request_id,
                RejectReason::PlayersNotReady { players: not_ready },
            );
        }

        // PlayerId 在大厅内必须保持稳定；只在确定能够开局、且即将给每位玩家
        // 下发包含新 `you` 的私有牌局快照时，才压缩并按座位重排参与者。
        self.remove_departed_players();

        let deck = self
            .shuffled_deck
            .take()
            .expect("a validated deck exists until the game starts");
        self.players.sort_by_key(|player| {
            player
                .seat
                .expect("all players selected a seat before starting")
                .0
        });
        for (index, player) in self.players.iter_mut().enumerate() {
            player.id = PlayerId(index as u8);
            player.ready = false;
            player.auto_play = player.is_bot;
        }
        let game_rules = RuleSet {
            player_count: self.players.len() as u8,
            ..self.rules
        };
        #[cfg(not(feature = "developer"))]
        let game = GameState::new_with_deck(game_rules, deck)
            .expect("host validated rules and deck during construction");
        #[cfg(feature = "developer")]
        let game = {
            let mut deck = deck;
            if game_rules.developer_deck {
                deck.truncate(
                    usize::from(game_rules.player_count) * usize::from(game_rules.hand_size),
                );
                GameState::new_with_development_deck(game_rules, deck)
                    .expect("the development deck is a valid shuffled subset")
            } else {
                GameState::new_with_deck(game_rules, deck)
                    .expect("host validated rules and deck during construction")
            }
        };
        self.game = Some(game);
        self.match_id = Some(new_match_id());
        self.finished_reference_changes = None;
        self.match_profile_stats = vec![QiGui523ProfileStats::default(); self.players.len()];
        self.auto_play_delay = None;
        self.reset_auto_play_delay_for_current_turn();
        self.initialize_turn_timer();
        self.bump_revision();
        self.broadcast_game(Some((connection, request_id)))
    }

    fn return_to_lobby(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
    ) -> Vec<Delivery> {
        if self.player_id(connection).is_none() {
            return self.reject(connection, request_id, RejectReason::NotJoined);
        }
        if self.host_connection != Some(connection) {
            return self.reject(
                connection,
                request_id,
                RejectReason::OnlyHostCanReturnToLobby,
            );
        }
        let Some(game) = self.game.as_ref() else {
            return self.reject(connection, request_id, RejectReason::GameNotStarted);
        };
        if !matches!(game.phase(), Phase::Finished(_)) {
            return self.reject(connection, request_id, RejectReason::GameNotFinished);
        }

        self.game = None;
        self.match_id = None;
        self.finished_reference_changes = None;
        self.match_profile_stats.clear();
        self.turn_timer = None;
        self.auto_play_delay = None;
        #[cfg(feature = "developer")]
        self.room.remove_developer_bots();
        let host_connection = self.host_connection;
        for player in &mut self.players {
            player.ready = host_connection == Some(player.connection);
            player.auto_play = false;
            if !player.connected {
                player.seat = None;
            }
        }
        let mut deck = build_deck(self.rules.deck_count);
        fastrand::shuffle(&mut deck);
        self.shuffled_deck = Some(deck);
        self.bump_revision();
        self.broadcast_lobby(Some((connection, request_id)))
    }

    fn play_again(&mut self, connection: ConnectionId, request_id: RequestId) -> Vec<Delivery> {
        let Some(player) = self.player_id(connection) else {
            return self.reject(connection, request_id, RejectReason::NotJoined);
        };
        let Some(game) = self.game.as_ref() else {
            return self.reject(connection, request_id, RejectReason::GameNotStarted);
        };
        if !matches!(game.phase(), Phase::Finished(_)) {
            return self.reject(connection, request_id, RejectReason::GameNotFinished);
        }

        let participant = &mut self.players[usize::from(player.0)];
        if !participant.ready {
            participant.ready = true;
            self.bump_revision();
        }

        let active_players = self.players.iter().filter(|player| !player.left);
        let active_count = active_players.clone().count();
        if active_count >= 2 && active_players.clone().all(|player| player.ready) {
            return self.start_next_game(Some((connection, request_id)));
        }
        self.broadcast_game(Some((connection, request_id)))
    }

    fn leave_room(&mut self, connection: ConnectionId, request_id: RequestId) -> Vec<Delivery> {
        let Some(index) = self
            .players
            .iter()
            .position(|player| player.connection == connection && !player.left)
        else {
            return self.reject(connection, request_id, RejectReason::NotJoined);
        };
        if self.host_connection == Some(connection) {
            return self.close_room(connection, request_id);
        }

        let name = self.players[index].name.clone();
        self.players[index].ready = false;
        self.players[index].connected = false;
        self.players[index].left = true;
        let automatic_effect = if self.current_player_is_disconnected() {
            let effect = self.play_automatic_action();
            self.reset_timer_for_current_turn();
            effect
        } else {
            None
        };
        self.bump_revision();

        let mut deliveries = automatic_effect
            .map(|effect| self.broadcast_play_effect(effect))
            .unwrap_or_default();
        deliveries.push(self.delivery(connection, Some(request_id), ServerEvent::LeftRoom));
        deliveries.extend(
            self.players
                .iter()
                .filter(|player| player.connected && !player.left)
                .map(|player| {
                    self.delivery(
                        player.connection,
                        None,
                        ServerEvent::PlayerLeft { name: name.clone() },
                    )
                }),
        );
        let should_start_next = self
            .game
            .as_ref()
            .is_some_and(|game| matches!(game.phase(), Phase::Finished(_)))
            && self.players.iter().filter(|player| !player.left).count() >= 2
            && self
                .players
                .iter()
                .filter(|player| !player.left)
                .all(|player| player.ready);
        if should_start_next {
            deliveries.extend(self.start_next_game(None));
        } else if self.game.is_some() {
            deliveries.extend(self.broadcast_game(None));
        } else {
            deliveries.extend(self.broadcast_lobby(None));
        }
        deliveries
    }

    fn start_next_game(&mut self, origin: Option<(ConnectionId, RequestId)>) -> Vec<Delivery> {
        self.game = None;
        self.turn_timer = None;
        self.auto_play_delay = None;
        self.remove_departed_players();
        for player in &mut self.players {
            player.ready = false;
        }

        let mut deck = build_deck(self.rules.deck_count);
        fastrand::shuffle(&mut deck);
        let game_rules = RuleSet {
            player_count: self.players.len() as u8,
            ..self.rules
        };
        #[cfg(not(feature = "developer"))]
        let game = GameState::new_with_deck(game_rules, deck)
            .expect("a freshly built deck always matches the configured rules");
        #[cfg(feature = "developer")]
        let game = {
            let mut deck = deck;
            if game_rules.developer_deck {
                deck.truncate(
                    usize::from(game_rules.player_count) * usize::from(game_rules.hand_size),
                );
                GameState::new_with_development_deck(game_rules, deck)
                    .expect("the development deck is a valid shuffled subset")
            } else {
                GameState::new_with_deck(game_rules, deck)
                    .expect("a freshly built deck always matches the configured rules")
            }
        };
        self.game = Some(game);
        self.match_id = Some(new_match_id());
        self.finished_reference_changes = None;
        self.match_profile_stats = vec![QiGui523ProfileStats::default(); self.players.len()];
        self.reset_auto_play_delay_for_current_turn();
        self.initialize_turn_timer();
        self.bump_revision();
        self.broadcast_game(origin)
    }

    fn close_room(&mut self, connection: ConnectionId, request_id: RequestId) -> Vec<Delivery> {
        self.room
            .close_room(connection, request_id)
            .unwrap_or_else(|reason| self.reject(connection, request_id, reason))
    }

    fn play_cards(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        cards: &[Card],
    ) -> Vec<Delivery> {
        let Some(player) = self.player_id(connection) else {
            return self.reject(connection, request_id, RejectReason::NotJoined);
        };
        let Some(game) = self.game.as_mut() else {
            return self.reject(connection, request_id, RejectReason::GameNotStarted);
        };

        match game.play_cards(to_core_player(player), cards) {
            Ok(_) => {
                let play = classify(cards, &self.rules)
                    .expect("the game accepted a play that the shared classifier recognizes");
                let effect = (
                    player,
                    PublicPlay {
                        kind: play.kind().clone(),
                        cards: cards.to_vec(),
                    },
                );
                self.reset_timer_for_current_turn();
                self.bump_revision();
                self.broadcast_game_after_action(Some((connection, request_id)), Some(effect))
            }
            Err(error) => self.reject(
                connection,
                request_id,
                RejectReason::GameViolation(GameViolation::QiGui523(map_game_error(&error))),
            ),
        }
    }

    fn pass(&mut self, connection: ConnectionId, request_id: RequestId) -> Vec<Delivery> {
        let Some(player) = self.player_id(connection) else {
            return self.reject(connection, request_id, RejectReason::NotJoined);
        };
        let Some(game) = self.game.as_mut() else {
            return self.reject(connection, request_id, RejectReason::GameNotStarted);
        };

        match game.pass(to_core_player(player)) {
            Ok(_) => {
                self.reset_timer_for_current_turn();
                self.bump_revision();
                self.broadcast_game_after_update(Some((connection, request_id)))
            }
            Err(error) => self.reject(
                connection,
                request_id,
                RejectReason::GameViolation(GameViolation::QiGui523(map_game_error(&error))),
            ),
        }
    }

    fn initialize_turn_timer(&mut self) {
        if self.rules.time_control.is_unlimited() {
            self.turn_timer = None;
            return;
        }
        let Some(game) = self.game.as_ref() else {
            self.turn_timer = None;
            return;
        };
        let Some(current) = game
            .trick()
            .map(|trick| from_core_player(trick.current_player()))
        else {
            self.turn_timer = None;
            return;
        };
        let control = self.rules.time_control;
        self.turn_timer = Some(TurnTimerState {
            player: current,
            base_remaining: Duration::from_secs(u64::from(control.base_seconds())),
            reserve_remaining: vec![
                Duration::from_secs(u64::from(control.reserve_seconds()));
                game.players().len()
            ],
        });
    }

    fn reset_timer_for_current_turn(&mut self) {
        self.reset_auto_play_delay_for_current_turn();
        if self.rules.time_control.is_unlimited() {
            self.turn_timer = None;
            return;
        }
        let Some(current) = self.game.as_ref().and_then(|game| {
            matches!(game.phase(), Phase::Playing)
                .then(|| {
                    game.trick()
                        .map(|trick| from_core_player(trick.current_player()))
                })
                .flatten()
        }) else {
            self.turn_timer = None;
            return;
        };
        let base = Duration::from_secs(u64::from(self.rules.time_control.base_seconds()));
        match self.turn_timer.as_mut() {
            Some(timer) => {
                timer.player = current;
                timer.base_remaining = base;
            }
            None => self.initialize_turn_timer(),
        }
    }

    fn turn_timer_view(&self) -> Option<TurnTimerView> {
        let timer = self.turn_timer.as_ref()?;
        Some(TurnTimerView {
            player: timer.player,
            base_seconds: duration_ceil_seconds(timer.base_remaining),
            reserve_seconds: duration_ceil_seconds(
                timer.reserve_remaining[usize::from(timer.player.0)],
            ),
        })
    }

    fn current_player(&self) -> Option<PlayerId> {
        self.game.as_ref().and_then(|game| {
            matches!(game.phase(), Phase::Playing)
                .then(|| {
                    game.trick()
                        .map(|trick| from_core_player(trick.current_player()))
                })
                .flatten()
        })
    }

    fn current_player_is_disconnected(&self) -> bool {
        let Some(current) = self.current_player() else {
            return false;
        };
        self.players
            .iter()
            .find(|player| player.id == current)
            .is_some_and(|player| !player.connected && !player.is_bot)
    }

    fn current_auto_play_player(&self) -> Option<PlayerId> {
        let current = self.current_player()?;
        self.players
            .iter()
            .find(|player| player.id == current)
            .is_some_and(|player| (player.connected || player.is_bot) && player.auto_play)
            .then_some(current)
    }

    fn reset_auto_play_delay_for_current_turn(&mut self) {
        self.auto_play_delay = self
            .current_auto_play_player()
            .map(|player| AutoPlayDelayState {
                player,
                remaining: AUTO_PLAY_DELAY,
            });
    }

    fn play_automatic_action(&mut self) -> Option<(PlayerId, PublicPlay)> {
        let (player, cards) = {
            let game = self.game.as_ref().expect("a running timer has a game");
            let trick = game.trick().expect("a playing game has a trick");
            let player = trick.current_player();
            let cards = if let Some(current_play) = trick.winning_play() {
                let played_cards = trick
                    .records()
                    .iter()
                    .flat_map(|record| match record {
                        PlayRecord::Played { play, .. } => play.cards(),
                        PlayRecord::Passed { .. } => &[],
                    })
                    .copied()
                    .collect::<Vec<_>>();
                QiGui523Bot::new()
                    .choose(QiGui523BotRequest {
                        hand: game.player(player).expect("current player exists").hand(),
                        current_play,
                        played_cards: &played_cards,
                        rules: game.rules(),
                    })
                    .map(|play| play.cards().to_vec())
            } else {
                game.player(player)
                    .expect("current player exists")
                    .hand()
                    .iter()
                    .copied()
                    .min_by_key(|card| {
                        (card.rank().strength(), card.suit().strength(), card.deck())
                    })
                    .map(|card| vec![card])
            };
            (player, cards)
        };

        let game = self.game.as_mut().expect("a running timer has a game");
        if let Some(cards) = cards {
            let play = classify(&cards, game.rules())
                .expect("the timeout strategy only returns classifiable cards");
            let effect = PublicPlay {
                kind: play.kind().clone(),
                cards: cards.clone(),
            };
            game.play_cards(player, &cards)
                .expect("the timeout strategy only returns legal cards");
            Some((from_core_player(player), effect))
        } else {
            game.pass(player)
                .expect("a player without a legal response may pass");
            None
        }
    }

    fn snapshot(&self, connection: ConnectionId, request_id: RequestId) -> Vec<Delivery> {
        let Some(player) = self.player_id(connection) else {
            return self.reject(connection, request_id, RejectReason::NotJoined);
        };
        let event = if self.game.is_some() {
            ServerEvent::GameSnapshot(GameSnapshot::QiGui523(self.game_snapshot(player)))
        } else {
            ServerEvent::LobbySnapshot(self.lobby_snapshot())
        };
        vec![self.delivery(connection, Some(request_id), event)]
    }

    fn broadcast_lobby(&self, origin: Option<(ConnectionId, RequestId)>) -> Vec<Delivery> {
        self.room
            .broadcast_lobby(GameKind::QiGui523, GameRules::QiGui523(self.rules), origin)
    }

    fn broadcast_game(&self, origin: Option<(ConnectionId, RequestId)>) -> Vec<Delivery> {
        self.players
            .iter()
            .filter(|player| player.connected)
            .map(|player| {
                let reply = origin
                    .filter(|(connection, _)| *connection == player.connection)
                    .map(|(_, request)| request);
                self.delivery(
                    player.connection,
                    reply,
                    ServerEvent::GameSnapshot(GameSnapshot::QiGui523(
                        self.game_snapshot(player.id),
                    )),
                )
            })
            .collect()
    }

    fn broadcast_game_after_update(
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
            return self
                .players
                .iter()
                .filter(|player| player.connected && !player.left)
                .map(|player| self.delivery(player.connection, None, ServerEvent::RoomClosed))
                .collect();
        }

        let mut deliveries = Vec::new();
        for (_, name) in departed {
            deliveries.extend(
                self.players
                    .iter()
                    .filter(|player| player.connected && !player.left)
                    .map(|player| {
                        self.delivery(
                            player.connection,
                            None,
                            ServerEvent::PlayerLeft { name: name.clone() },
                        )
                    }),
            );
        }
        deliveries.extend(self.broadcast_game(origin));
        deliveries
    }

    fn apply_finished_reference_points(&mut self) {
        if self.finished_reference_changes.is_some() {
            return;
        }
        let Some(scores) = self.game.as_ref().and_then(|game| match game.phase() {
            Phase::Finished(result) => Some(result.scores.clone()),
            Phase::Playing => None,
        }) else {
            return;
        };
        let deltas = reference_point_deltas(&scores)
            .expect("a running game always contains between two and six players");
        let mut changes = Vec::with_capacity(self.players.len());
        let match_profile_stats = self.match_profile_stats.clone();
        self.room.prepare_rematch();
        for (index, ((player, delta), score)) in self
            .players
            .iter_mut()
            .zip(deltas)
            .zip(scores.iter().copied())
            .enumerate()
        {
            player.reference_points = player.reference_points.saturating_add(i32::from(delta));
            player.completed_games = player.completed_games.saturating_add(1);
            let placement = 1 + scores.iter().filter(|other| **other > score).count();
            let aggregate = player
                .game_profiles
                .qigui523
                .get_or_insert_with(QiGui523ProfileStats::default);
            aggregate.completed_games = aggregate.completed_games.saturating_add(1);
            aggregate.total_score = aggregate.total_score.saturating_add(u64::from(score));
            aggregate.total_reference_delta = aggregate
                .total_reference_delta
                .saturating_add(i64::from(delta));
            if let Some(count) = aggregate.placement_counts.get_mut(placement - 1) {
                *count = count.saturating_add(1);
            }
            if let Some(current) = match_profile_stats.get(index) {
                merge_qigui523_play_stats(aggregate, current);
            }
            changes.push(PlayerReferenceChange {
                player: player.id,
                profile_id: player.profile_id,
                delta,
            });
        }
        self.finished_reference_changes = Some(changes);
    }

    fn broadcast_game_after_action(
        &mut self,
        origin: Option<(ConnectionId, RequestId)>,
        effect: Option<(PlayerId, PublicPlay)>,
    ) -> Vec<Delivery> {
        if let Some((player, play)) = effect.as_ref()
            && let Some(stats) = self.match_profile_stats.get_mut(usize::from(player.0))
        {
            record_qigui523_play(stats, &play.kind);
        }
        let mut deliveries = effect
            .map(|effect| self.broadcast_play_effect(effect))
            .unwrap_or_default();
        deliveries.extend(self.broadcast_game_after_update(origin));
        deliveries
    }

    fn broadcast_play_effect(&self, effect: (PlayerId, PublicPlay)) -> Vec<Delivery> {
        let (player, play) = effect;
        self.players
            .iter()
            .filter(|participant| participant.connected && !participant.left)
            .map(|participant| {
                self.delivery(
                    participant.connection,
                    None,
                    ServerEvent::GameEvent(GameEvent::QiGui523(QiGui523Event::PlayEffect {
                        player,
                        play: play.clone(),
                    })),
                )
            })
            .collect()
    }

    fn lobby_snapshot(&self) -> LobbySnapshot {
        self.room
            .lobby_snapshot(GameKind::QiGui523, GameRules::QiGui523(self.rules))
    }

    fn game_snapshot(&self, recipient: PlayerId) -> QiGui523Snapshot {
        let game = self.game.as_ref().expect("game snapshot requires a game");
        let recipient_index = usize::from(recipient.0);
        let players = game
            .players()
            .iter()
            .zip(&self.players)
            .map(|(state, participant)| PlayerPublicState {
                id: participant.id,
                profile_id: participant.profile_id,
                name: participant.name.clone(),
                avatar: participant.avatar,
                seat: participant
                    .seat
                    .expect("all game participants have selected seats"),
                hand_len: state.hand().len() as u16,
                score: state.score(),
                ready: participant.ready,
                connected: participant.connected || participant.is_bot,
                auto_play: participant.auto_play,
                reference_points: participant.reference_points,
                completed_games: participant.completed_games,
                game_profiles: participant.game_profiles.clone(),
            })
            .collect();
        let starting = game.starting_card();

        QiGui523Snapshot {
            match_id: self.match_id.expect("a running game has a match id"),
            host_port: self.host_port,
            you: recipient,
            host: self
                .host_connection
                .and_then(|connection| {
                    self.players
                        .iter()
                        .find(|player| player.connection == connection)
                        .map(|player| player.id)
                })
                .expect("a running game has a room host"),
            players,
            your_hand: game.players()[recipient_index].hand().to_vec(),
            draw_pile_len: game.draw_pile_len() as u16,
            starting_card: StartingCardView {
                player: from_core_player(starting.player),
                card: starting.card,
            },
            trick: game.trick().map(|trick| TrickView {
                leader: from_core_player(trick.leader()),
                current_player: from_core_player(trick.current_player()),
                winning_player: trick.winning_player().map(from_core_player),
                winning_play: trick.winning_play().map(|play| PublicPlay {
                    kind: play.kind().clone(),
                    cards: play.cards().to_vec(),
                }),
                records: trick
                    .records()
                    .iter()
                    .map(|record| match record {
                        PlayRecord::Played { player, play } => PublicPlayRecord::Played {
                            player: from_core_player(*player),
                            play: PublicPlay {
                                kind: play.kind().clone(),
                                cards: play.cards().to_vec(),
                            },
                        },
                        PlayRecord::Passed { player } => PublicPlayRecord::Passed {
                            player: from_core_player(*player),
                        },
                    })
                    .collect(),
                table_points: trick.table_points(),
            }),
            turn_timer: self.turn_timer_view(),
            phase: match game.phase() {
                Phase::Playing => GamePhaseView::Playing,
                Phase::Finished(result) => GamePhaseView::Finished {
                    match_id: self.match_id.expect("a running game has a match id"),
                    finisher: from_core_player(result.finisher),
                    scores: result
                        .scores
                        .iter()
                        .enumerate()
                        .map(|(index, score)| PlayerScore {
                            player: PlayerId(index as u8),
                            score: *score,
                        })
                        .collect(),
                    remaining_hands: game
                        .players()
                        .iter()
                        .enumerate()
                        .map(|(index, player)| RevealedHand {
                            player: PlayerId(index as u8),
                            cards: player.hand().to_vec(),
                        })
                        .collect(),
                    reference_changes: self
                        .finished_reference_changes
                        .clone()
                        .expect("finished points are applied before broadcasting a snapshot"),
                    captured_hand_points: result.captured_hand_points,
                },
            },
        }
    }
}

fn validate_deck(deck_count: u8, deck: &[Card]) -> Result<(), HostError> {
    let expected_deck = build_deck(deck_count);
    if deck.len() != expected_deck.len() {
        return Err(HostError::InvalidDeckSize {
            expected: expected_deck.len(),
            actual: deck.len(),
        });
    }
    let expected: HashSet<_> = expected_deck.into_iter().collect();
    let actual: HashSet<_> = deck.iter().copied().collect();
    if actual.len() != deck.len() || actual != expected {
        return Err(HostError::InvalidDeckContents);
    }
    Ok(())
}

fn record_qigui523_play(stats: &mut QiGui523ProfileStats, kind: &PlayKind) {
    match kind {
        PlayKind::Straight { card_count } => {
            stats.straight_plays = stats.straight_plays.saturating_add(1);
            stats.longest_straight = stats
                .longest_straight
                .max(u16::try_from(*card_count).unwrap_or(u16::MAX));
        }
        PlayKind::ConsecutivePairs { pair_count } => {
            stats.consecutive_pair_plays = stats.consecutive_pair_plays.saturating_add(1);
            stats.longest_consecutive_pairs = stats
                .longest_consecutive_pairs
                .max(u16::try_from(*pair_count).unwrap_or(u16::MAX));
        }
        PlayKind::Airplane { triple_count } => {
            stats.airplane_plays = stats.airplane_plays.saturating_add(1);
            stats.longest_airplane = stats
                .longest_airplane
                .max(u16::try_from(*triple_count).unwrap_or(u16::MAX));
        }
        PlayKind::Bomb(_) => stats.bomb_plays = stats.bomb_plays.saturating_add(1),
        PlayKind::HeavenBomb => {
            stats.heaven_bomb_plays = stats.heaven_bomb_plays.saturating_add(1);
        }
        PlayKind::Single
        | PlayKind::Pair
        | PlayKind::Triple
        | PlayKind::TripleWithSingle
        | PlayKind::TripleWithPair => {}
    }
}

fn merge_qigui523_play_stats(aggregate: &mut QiGui523ProfileStats, current: &QiGui523ProfileStats) {
    aggregate.straight_plays = aggregate
        .straight_plays
        .saturating_add(current.straight_plays);
    aggregate.consecutive_pair_plays = aggregate
        .consecutive_pair_plays
        .saturating_add(current.consecutive_pair_plays);
    aggregate.airplane_plays = aggregate
        .airplane_plays
        .saturating_add(current.airplane_plays);
    aggregate.bomb_plays = aggregate.bomb_plays.saturating_add(current.bomb_plays);
    aggregate.heaven_bomb_plays = aggregate
        .heaven_bomb_plays
        .saturating_add(current.heaven_bomb_plays);
    aggregate.longest_straight = aggregate.longest_straight.max(current.longest_straight);
    aggregate.longest_consecutive_pairs = aggregate
        .longest_consecutive_pairs
        .max(current.longest_consecutive_pairs);
    aggregate.longest_airplane = aggregate.longest_airplane.max(current.longest_airplane);
}

fn to_core_player(player: PlayerId) -> CorePlayerId {
    CorePlayerId(usize::from(player.0))
}

fn from_core_player(player: CorePlayerId) -> PlayerId {
    PlayerId(u8::try_from(player.0).expect("core supports at most six players"))
}

fn map_game_error(error: &GameError) -> RuleViolation {
    match error {
        GameError::InvalidPlayer(_) => RuleViolation::InvalidPlayer,
        GameError::NotPlayersTurn { .. } => RuleViolation::NotPlayersTurn,
        GameError::GameAlreadyFinished => RuleViolation::GameAlreadyFinished,
        GameError::MustLeadWithCards => RuleViolation::MustLeadWithCards,
        GameError::CardNotInHand(_) => RuleViolation::CardNotInHand,
        GameError::InvalidPlay(
            PlayError::Empty
            | PlayError::DuplicatePhysicalCard(_)
            | PlayError::CardOutsideConfiguredDeck(_)
            | PlayError::InvalidPattern,
        )
        | GameError::InvalidRules(_)
        | GameError::InvalidDeckSize { .. }
        | GameError::InvalidDeckContents => RuleViolation::InvalidPattern,
        GameError::PlayDoesNotBeatCurrent => RuleViolation::PlayDoesNotBeatCurrent,
    }
}

fn duration_ceil_seconds(duration: Duration) -> u16 {
    let milliseconds = duration.as_millis();
    u16::try_from(milliseconds.div_ceil(1_000)).unwrap_or(u16::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use leocard_protocol::{
        AVATAR_DIMENSION, ClientCommand, QUICK_VOICE_COUNT, RuleViolation, join_identity_payload,
    };
    use leocard_qigui523::{can_beat, classify};
    use std::collections::HashMap;
    use std::io::Cursor;

    const ROOM: RoomId = RoomId(523);
    const HOST: ConnectionId = ConnectionId(10);
    const SECOND: ConnectionId = ConnectionId(20);
    const THIRD: ConnectionId = ConnectionId(30);

    fn qigui523_snapshot(event: &ServerEvent) -> Option<&QiGui523Snapshot> {
        let ServerEvent::GameSnapshot(snapshot) = event else {
            return None;
        };
        snapshot.qigui523()
    }

    fn qigui523_play_effect(event: &ServerEvent) -> Option<(PlayerId, &PublicPlay)> {
        let ServerEvent::GameEvent(GameEvent::QiGui523(QiGui523Event::PlayEffect { player, play })) =
            event
        else {
            return None;
        };
        Some((*player, play))
    }

    fn rules() -> RuleSet {
        RuleSet {
            player_count: 6,
            ..RuleSet::default()
        }
    }

    fn message(connection_request: u64, command: ClientCommand) -> ClientMessage {
        ClientMessage::new(ROOM, RequestId(connection_request), command)
    }

    fn join_command(name: &str, token: ReconnectToken) -> ClientCommand {
        use ed25519_dalek::{Signer, SigningKey};

        let mut secret = [0; 32];
        secret[..8].copy_from_slice(&token.0.to_be_bytes());
        secret[8] = 1;
        let key = SigningKey::from_bytes(&secret);
        let game_profiles = PlayerGameProfiles::default();
        let payload = join_identity_payload(ROOM, token, name, 0, 0, &game_profiles);
        ClientCommand::Join {
            name: name.to_owned(),
            reconnect_token: token,
            profile_id: ProfileId(key.verifying_key().to_bytes()),
            reference_points: 0,
            completed_games: 0,
            game_profiles,
            identity_signature: key.sign(&payload).to_bytes().to_vec(),
        }
    }

    fn join_three(session: &mut QiGui523Session) {
        for (seat, (connection, name)) in [(HOST, "房主"), (SECOND, "玩家二"), (THIRD, "玩家三")]
            .into_iter()
            .enumerate()
        {
            session.handle(
                connection,
                message(1, join_command(name, ReconnectToken(connection.0))),
            );
            session.handle(
                connection,
                message(
                    2,
                    ClientCommand::SelectSeat {
                        seat: SeatId(seat as u8),
                    },
                ),
            );
        }
    }

    fn ready_and_start(session: &mut QiGui523Session) -> Vec<Delivery> {
        for connection in [HOST, SECOND, THIRD] {
            session.handle(
                connection,
                message(3, ClientCommand::SetReady { ready: true }),
            );
        }
        session.handle(HOST, message(4, ClientCommand::StartGame))
    }

    #[cfg(feature = "developer")]
    #[test]
    fn developer_player_can_replace_only_their_own_hand() {
        let mut session = QiGui523Session::new(ROOM, rules(), build_deck(1)).unwrap();
        join_three(&mut session);
        ready_and_start(&mut session);
        let player = session.player_id(SECOND).unwrap();
        let other = session.player_id(THIRD).unwrap();
        let other_hand = session
            .game()
            .unwrap()
            .player(to_core_player(other))
            .unwrap()
            .hand()
            .to_vec();
        let replacement = vec![
            Card::suited(
                0,
                leocard_qigui523::Suit::Club,
                leocard_qigui523::Rank::Joker,
            ),
            Card::suited(
                0,
                leocard_qigui523::Suit::Club,
                leocard_qigui523::Rank::Joker,
            ),
            Card::suited(
                9,
                leocard_qigui523::Suit::Spade,
                leocard_qigui523::Rank::Seven,
            ),
        ];

        let deliveries = send_next(
            &mut session,
            SECOND,
            ClientCommand::Game(GameCommand::QiGui523(QiGui523Command::SetDeveloperHand {
                cards: replacement.clone(),
            })),
        );

        assert_eq!(deliveries.len(), 3);
        let hand = session
            .game()
            .unwrap()
            .player(to_core_player(player))
            .unwrap()
            .hand();
        assert_eq!(hand.len(), replacement.len());
        assert_eq!(
            hand.iter().filter(|card| **card == replacement[0]).count(),
            2
        );
        assert!(hand.contains(&replacement[2]));
        assert_eq!(
            session
                .game()
                .unwrap()
                .player(to_core_player(other))
                .unwrap()
                .hand(),
            other_hand
        );
    }

    #[cfg(feature = "developer")]
    #[test]
    fn developer_player_can_play_a_semantic_pair_from_impossible_physical_copies() {
        let mut session = QiGui523Session::new(ROOM, rules(), build_deck(1)).unwrap();
        join_three(&mut session);
        ready_and_start(&mut session);

        let current = from_core_player(session.game().unwrap().trick().unwrap().current_player());
        let connection = connection_for_player(&session, current);
        let pair = vec![
            Card::suited(
                0,
                leocard_qigui523::Suit::Club,
                leocard_qigui523::Rank::Joker,
            ),
            Card::suited(
                1,
                leocard_qigui523::Suit::Club,
                leocard_qigui523::Rank::Joker,
            ),
        ];

        send_next(
            &mut session,
            connection,
            ClientCommand::Game(GameCommand::QiGui523(QiGui523Command::SetDeveloperHand {
                cards: pair.clone(),
            })),
        );
        let deliveries = send_next(
            &mut session,
            connection,
            ClientCommand::Game(GameCommand::QiGui523(QiGui523Command::PlayCards {
                cards: pair,
            })),
        );

        assert!(
            deliveries
                .iter()
                .all(|delivery| !matches!(delivery.message.event, ServerEvent::Rejected { .. }))
        );
        assert!(matches!(
            session.game().unwrap().phase(),
            Phase::Finished(_)
        ));
    }

    #[cfg(not(feature = "developer"))]
    #[test]
    fn normal_build_rejects_developer_hand_commands() {
        let mut session = QiGui523Session::new(ROOM, rules(), build_deck(1)).unwrap();
        join_three(&mut session);
        ready_and_start(&mut session);

        let rejected = send_next(
            &mut session,
            SECOND,
            ClientCommand::Game(GameCommand::QiGui523(QiGui523Command::SetDeveloperHand {
                cards: vec![Card::suited(
                    0,
                    leocard_qigui523::Suit::Spade,
                    leocard_qigui523::Rank::Seven,
                )],
            })),
        );
        assert_eq!(
            rejection(&rejected),
            Some(&RejectReason::DeveloperFeatureUnavailable)
        );
    }

    fn send_next(
        session: &mut QiGui523Session,
        connection: ConnectionId,
        command: ClientCommand,
    ) -> Vec<Delivery> {
        let request = session
            .last_requests
            .get(&connection)
            .map_or(1, |request| request.0 + 1);
        session.handle(connection, message(request, command))
    }

    fn connection_for_player(session: &QiGui523Session, player: PlayerId) -> ConnectionId {
        session
            .players
            .iter()
            .find(|participant| participant.id == player)
            .expect("player is connected")
            .connection
    }

    fn rejection(deliveries: &[Delivery]) -> Option<&RejectReason> {
        deliveries
            .iter()
            .find_map(|delivery| match &delivery.message.event {
                ServerEvent::Rejected { reason } => Some(reason),
                _ => None,
            })
    }

    fn avatar_png(width: u32, height: u32) -> Vec<u8> {
        let image = image::DynamicImage::ImageRgba8(image::ImageBuffer::from_pixel(
            width,
            height,
            image::Rgba([30, 120, 200, 255]),
        ));
        let mut output = Cursor::new(Vec::new());
        image
            .write_to(&mut output, image::ImageFormat::Png)
            .unwrap();
        output.into_inner()
    }

    fn finish_game(session: &mut QiGui523Session) {
        for _ in 0..1_000 {
            if matches!(session.game().unwrap().phase(), Phase::Finished(_)) {
                return;
            }
            let (current, playable) = {
                let game = session.game().unwrap();
                let trick = game.trick().unwrap();
                let current = trick.current_player();
                let winning_play = trick.winning_play();
                let playable = game
                    .player(current)
                    .unwrap()
                    .hand()
                    .iter()
                    .copied()
                    .find(|card| {
                        let candidate = classify(&[*card], game.rules()).unwrap();
                        winning_play
                            .is_none_or(|winning| can_beat(&candidate, winning, game.rules()))
                    });
                (current, playable)
            };
            let game = session.game.as_mut().unwrap();
            if let Some(card) = playable {
                game.play_cards(current, &[card]).unwrap();
            } else {
                game.pass(current).unwrap();
            }
        }
        panic!("deterministic game did not terminate");
    }

    #[test]
    fn player_name_limit_counts_unicode_characters() {
        let mut accepted = QiGui523Session::new(ROOM, rules(), build_deck(1)).unwrap();
        let deliveries = accepted.handle(
            HOST,
            message(1, join_command("一二三四五六七", ReconnectToken(HOST.0))),
        );
        assert_eq!(rejection(&deliveries), None);

        let mut rejected = QiGui523Session::new(ROOM, rules(), build_deck(1)).unwrap();
        let deliveries = rejected.handle(
            HOST,
            message(1, join_command("一二三四五六七八", ReconnectToken(HOST.0))),
        );
        assert_eq!(
            rejection(&deliveries),
            Some(&RejectReason::NameTooLong {
                max_chars: MAX_PLAYER_NAME_CHARS as u16,
            })
        );
    }

    #[test]
    fn invalid_player_identity_signature_is_rejected() {
        let mut session = QiGui523Session::new(ROOM, rules(), build_deck(1)).unwrap();
        let mut command = join_command("玩家", ReconnectToken(HOST.0));
        let ClientCommand::Join {
            identity_signature, ..
        } = &mut command
        else {
            unreachable!()
        };
        identity_signature[0] ^= 0x80;

        let deliveries = session.handle(HOST, message(1, command));

        assert_eq!(
            rejection(&deliveries),
            Some(&RejectReason::InvalidIdentityProof)
        );
        assert!(session.players.is_empty());
    }

    #[test]
    fn finished_match_applies_reference_points_exactly_once() {
        let mut session = QiGui523Session::new(ROOM, rules(), build_deck(1)).unwrap();
        join_three(&mut session);
        ready_and_start(&mut session);
        finish_game(&mut session);
        let scores = match session.game().unwrap().phase() {
            Phase::Finished(result) => result.scores.clone(),
            Phase::Playing => unreachable!(),
        };
        let expected = reference_point_deltas(&scores).unwrap();
        let expected_remaining_hands = session
            .game()
            .unwrap()
            .players()
            .iter()
            .enumerate()
            .map(|(index, player)| (PlayerId(index as u8), player.hand().to_vec()))
            .collect::<Vec<_>>();

        let deliveries = session.broadcast_game_after_update(None);

        assert_eq!(
            session
                .players
                .iter()
                .map(|player| player.reference_points)
                .collect::<Vec<_>>(),
            expected
                .iter()
                .map(|delta| i32::from(*delta))
                .collect::<Vec<_>>()
        );
        assert!(
            session
                .players
                .iter()
                .all(|player| player.completed_games == 1)
        );
        for ((player, score), delta) in session
            .players
            .iter()
            .zip(scores.iter().copied())
            .zip(expected.iter().copied())
        {
            let stats = player.game_profiles.qigui523.as_ref().unwrap();
            assert_eq!(stats.completed_games, 1);
            assert_eq!(stats.total_score, u64::from(score));
            assert_eq!(stats.total_reference_delta, i64::from(delta));
            let placement = 1 + scores.iter().filter(|other| **other > score).count();
            assert_eq!(stats.placement_counts.iter().sum::<u32>(), 1);
            assert_eq!(stats.placement_counts[placement - 1], 1);
        }
        assert!(deliveries.iter().all(|delivery| {
            qigui523_snapshot(&delivery.message.event).is_some_and(|snapshot| {
                matches!(
                    &snapshot.phase,
                    GamePhaseView::Finished {
                        reference_changes,
                        remaining_hands,
                        ..
                    } if reference_changes.len() == expected.len()
                        && remaining_hands
                            .iter()
                            .map(|hand| (hand.player, hand.cards.clone()))
                            .collect::<Vec<_>>() == expected_remaining_hands
                )
            })
        }));

        session.broadcast_game_after_update(None);
        assert_eq!(
            session
                .players
                .iter()
                .map(|player| player.reference_points)
                .collect::<Vec<_>>(),
            expected
                .iter()
                .map(|delta| i32::from(*delta))
                .collect::<Vec<_>>()
        );
        assert!(
            session
                .players
                .iter()
                .all(|player| player.completed_games == 1)
        );
        assert!(session.players.iter().all(|player| {
            player
                .game_profiles
                .qigui523
                .as_ref()
                .is_some_and(|stats| stats.completed_games == 1)
        }));
    }

    #[test]
    fn qigui523_profile_play_statistics_count_types_and_keep_longest_lengths() {
        let mut stats = QiGui523ProfileStats::default();
        for kind in [
            PlayKind::Straight { card_count: 5 },
            PlayKind::Straight { card_count: 8 },
            PlayKind::ConsecutivePairs { pair_count: 3 },
            PlayKind::Airplane { triple_count: 2 },
            PlayKind::Bomb(leocard_qigui523::BombKind::OfAKind {
                card_count: 4,
                rank: leocard_qigui523::Rank::Ace,
            }),
            PlayKind::HeavenBomb,
        ] {
            record_qigui523_play(&mut stats, &kind);
        }

        assert_eq!(stats.straight_plays, 2);
        assert_eq!(stats.consecutive_pair_plays, 1);
        assert_eq!(stats.airplane_plays, 1);
        assert_eq!(stats.bomb_plays, 1);
        assert_eq!(stats.heaven_bomb_plays, 1);
        assert_eq!(stats.longest_straight, 8);
        assert_eq!(stats.longest_consecutive_pairs, 3);
        assert_eq!(stats.longest_airplane, 2);
    }

    #[test]
    fn only_host_can_close_room_and_every_connected_player_is_notified() {
        let mut session = QiGui523Session::new(ROOM, rules(), build_deck(1)).unwrap();
        join_three(&mut session);

        let denied = session.handle(SECOND, message(3, ClientCommand::CloseRoom));
        assert_eq!(
            rejection(&denied),
            Some(&RejectReason::OnlyHostCanCloseRoom)
        );
        assert!(!session.is_closed());

        let deliveries = session.handle(HOST, message(3, ClientCommand::CloseRoom));
        assert!(session.is_closed());
        assert_eq!(deliveries.len(), 3);
        assert!(
            deliveries
                .iter()
                .all(|delivery| { matches!(delivery.message.event, ServerEvent::RoomClosed) })
        );
    }

    #[test]
    fn unlimited_time_control_never_creates_or_advances_a_turn_timer() {
        let configured_rules = RuleSet {
            time_control: leocard_qigui523::TimeControl::Unlimited,
            ..rules()
        };
        let mut session = QiGui523Session::new(
            ROOM,
            configured_rules,
            build_deck(configured_rules.deck_count),
        )
        .unwrap();
        join_three(&mut session);
        ready_and_start(&mut session);
        let current = session.game().unwrap().trick().unwrap().current_player();

        assert!(session.turn_timer.is_none());
        assert!(session.advance_time(Duration::from_secs(600)).is_empty());
        assert_eq!(
            session.game().unwrap().trick().unwrap().current_player(),
            current
        );
    }

    #[test]
    fn turn_timer_spends_base_before_persistent_player_reserve() {
        let configured_rules = RuleSet {
            time_control: leocard_qigui523::TimeControl::FivePlusTen,
            ..rules()
        };
        let mut session = QiGui523Session::new(
            ROOM,
            configured_rules,
            build_deck(configured_rules.deck_count),
        )
        .unwrap();
        join_three(&mut session);
        ready_and_start(&mut session);

        let timed_player = session.turn_timer.as_ref().unwrap().player;
        assert_eq!(session.turn_timer_view().unwrap().base_seconds, 5);
        assert_eq!(session.turn_timer_view().unwrap().reserve_seconds, 10);

        session.advance_time(Duration::from_secs(5));
        assert_eq!(session.turn_timer_view().unwrap().base_seconds, 0);
        assert_eq!(session.turn_timer_view().unwrap().reserve_seconds, 10);
        session.advance_time(Duration::from_secs(3));
        assert_eq!(session.turn_timer_view().unwrap().reserve_seconds, 7);

        let card = session
            .game()
            .unwrap()
            .player(to_core_player(timed_player))
            .unwrap()
            .hand()[0];
        let connection = connection_for_player(&session, timed_player);
        send_next(
            &mut session,
            connection,
            ClientCommand::Game(GameCommand::QiGui523(QiGui523Command::PlayCards {
                cards: vec![card],
            })),
        );

        let timer = session.turn_timer.as_ref().unwrap();
        assert_eq!(
            timer.reserve_remaining[usize::from(timed_player.0)],
            Duration::from_secs(7)
        );
        assert_eq!(duration_ceil_seconds(timer.base_remaining), 5);
    }

    #[test]
    fn timeout_lead_plays_exactly_the_smallest_single_card() {
        let configured_rules = RuleSet {
            time_control: leocard_qigui523::TimeControl::FivePlusTen,
            ..rules()
        };
        let mut session = QiGui523Session::new(
            ROOM,
            configured_rules,
            build_deck(configured_rules.deck_count),
        )
        .unwrap();
        join_three(&mut session);
        ready_and_start(&mut session);

        let player = session.turn_timer.as_ref().unwrap().player;
        let expected = session
            .game()
            .unwrap()
            .player(to_core_player(player))
            .unwrap()
            .hand()
            .iter()
            .copied()
            .min_by_key(|card| (card.rank().strength(), card.suit().strength(), card.deck()))
            .unwrap();

        let deliveries = session.advance_time(Duration::from_secs(15));
        assert!(!deliveries.is_empty());
        let game = session.game().unwrap();
        assert!(matches!(
            &game.trick().unwrap().records()[0],
            PlayRecord::Played { player: record_player, play }
                if *record_player == to_core_player(player) && play.cards() == [expected]
        ));
        assert_eq!(session.turn_timer_view().unwrap().base_seconds, 5);
        assert_eq!(session.turn_timer_view().unwrap().reserve_seconds, 10);
    }

    #[test]
    fn enabling_auto_play_waits_one_second_before_acting_on_own_turn() {
        let mut session = QiGui523Session::new(ROOM, rules(), build_deck(1)).unwrap();
        join_three(&mut session);
        ready_and_start(&mut session);

        let current = from_core_player(session.game().unwrap().trick().unwrap().current_player());
        let connection = connection_for_player(&session, current);
        let hand_len_before = session
            .game()
            .unwrap()
            .player(to_core_player(current))
            .unwrap()
            .hand()
            .len();

        let enabled = send_next(
            &mut session,
            connection,
            ClientCommand::Game(GameCommand::QiGui523(QiGui523Command::SetAutoPlay {
                enabled: true,
            })),
        );

        assert!(session.players[usize::from(current.0)].auto_play);
        assert_eq!(
            session
                .game()
                .unwrap()
                .player(to_core_player(current))
                .unwrap()
                .hand()
                .len(),
            hand_len_before
        );
        assert!(!enabled.iter().any(|delivery| {
            qigui523_play_effect(&delivery.message.event)
                .is_some_and(|(player, _)| player == current)
        }));
        assert!(
            enabled
                .iter()
                .filter_map(|delivery| match &delivery.message.event {
                    ServerEvent::GameSnapshot(snapshot) => snapshot.qigui523(),
                    _ => None,
                })
                .all(|snapshot| snapshot
                    .players
                    .iter()
                    .find(|player| player.id == current)
                    .is_some_and(|player| player.auto_play))
        );

        assert!(session.advance_time(Duration::from_millis(999)).is_empty());
        assert_eq!(
            session
                .game()
                .unwrap()
                .player(to_core_player(current))
                .unwrap()
                .hand()
                .len(),
            hand_len_before
        );
        let acted = session.advance_time(Duration::from_millis(1));
        assert!(acted.iter().any(|delivery| {
            qigui523_play_effect(&delivery.message.event)
                .is_some_and(|(player, _)| player == current)
        }));
        assert_eq!(
            session
                .game()
                .unwrap()
                .player(to_core_player(current))
                .unwrap()
                .hand()
                .len(),
            hand_len_before - 1
        );

        let disabled = send_next(
            &mut session,
            connection,
            ClientCommand::Game(GameCommand::QiGui523(QiGui523Command::SetAutoPlay {
                enabled: false,
            })),
        );
        assert!(!session.players[usize::from(current.0)].auto_play);
        assert!(
            disabled
                .iter()
                .filter_map(|delivery| match &delivery.message.event {
                    ServerEvent::GameSnapshot(snapshot) => snapshot.qigui523(),
                    _ => None,
                })
                .all(|snapshot| snapshot
                    .players
                    .iter()
                    .find(|player| player.id == current)
                    .is_some_and(|player| !player.auto_play))
        );
    }

    #[test]
    fn auto_play_also_waits_one_second_when_its_turn_arrives_later() {
        let mut session = QiGui523Session::new(ROOM, rules(), build_deck(1)).unwrap();
        join_three(&mut session);
        ready_and_start(&mut session);

        let (leader, managed, lead_card) = {
            let game = session.game().unwrap();
            let leader = game.trick().unwrap().current_player();
            let managed =
                CorePlayerId((leader.0 + game.players().len() - 1) % game.players().len());
            let lead_card = game.player(leader).unwrap().hand()[0];
            (
                from_core_player(leader),
                from_core_player(managed),
                lead_card,
            )
        };
        let managed_connection = connection_for_player(&session, managed);
        send_next(
            &mut session,
            managed_connection,
            ClientCommand::Game(GameCommand::QiGui523(QiGui523Command::SetAutoPlay {
                enabled: true,
            })),
        );
        let leader_connection = connection_for_player(&session, leader);
        send_next(
            &mut session,
            leader_connection,
            ClientCommand::Game(GameCommand::QiGui523(QiGui523Command::PlayCards {
                cards: vec![lead_card],
            })),
        );
        assert_eq!(
            from_core_player(session.game().unwrap().trick().unwrap().current_player()),
            managed
        );
        assert_eq!(session.game().unwrap().trick().unwrap().records().len(), 1);

        assert!(session.advance_time(Duration::from_millis(999)).is_empty());
        assert_eq!(session.game().unwrap().trick().unwrap().records().len(), 1);
        assert!(!session.advance_time(Duration::from_millis(1)).is_empty());
        assert!(session.game().unwrap().trick().unwrap().records().len() > 1);
    }

    #[test]
    fn timeout_follow_uses_the_smallest_greedy_response() {
        let configured_rules = RuleSet {
            time_control: leocard_qigui523::TimeControl::FivePlusTen,
            ..rules()
        };
        let mut session = QiGui523Session::new(
            ROOM,
            configured_rules,
            build_deck(configured_rules.deck_count),
        )
        .unwrap();
        join_three(&mut session);
        ready_and_start(&mut session);

        let (leader, lead_card, follower, expected) = {
            let game = session.game().unwrap();
            let leader = game.trick().unwrap().current_player();
            let follower =
                CorePlayerId((leader.0 + game.players().len() - 1) % game.players().len());
            game.player(leader)
                .unwrap()
                .hand()
                .iter()
                .copied()
                .find_map(|lead_card| {
                    let current_play = classify(&[lead_card], game.rules()).unwrap();
                    let response = QiGui523Bot::new().choose(QiGui523BotRequest {
                        hand: game.player(follower).unwrap().hand(),
                        current_play: &current_play,
                        played_cards: &[lead_card],
                        rules: game.rules(),
                    })?;
                    Some((leader, lead_card, follower, response.cards().to_vec()))
                })
                .expect("deterministic hands contain a beatable single")
        };

        let leader = from_core_player(leader);
        let connection = connection_for_player(&session, leader);
        send_next(
            &mut session,
            connection,
            ClientCommand::Game(GameCommand::QiGui523(QiGui523Command::PlayCards {
                cards: vec![lead_card],
            })),
        );
        assert_eq!(
            session.turn_timer.as_ref().unwrap().player,
            from_core_player(follower)
        );

        session.advance_time(Duration::from_secs(15));
        let last_record = session
            .game()
            .unwrap()
            .trick()
            .unwrap()
            .records()
            .last()
            .unwrap();
        assert!(matches!(
            last_record,
            PlayRecord::Played { player, play }
                if *player == follower && play.cards() == expected
        ));
    }

    #[test]
    fn disconnected_current_player_is_replaced_immediately_without_spending_time() {
        let configured_rules = RuleSet {
            time_control: leocard_qigui523::TimeControl::FivePlusTen,
            ..rules()
        };
        let mut session = QiGui523Session::new(
            ROOM,
            configured_rules,
            build_deck(configured_rules.deck_count),
        )
        .unwrap();
        join_three(&mut session);
        ready_and_start(&mut session);

        if session
            .turn_timer
            .as_ref()
            .is_some_and(|timer| connection_for_player(&session, timer.player) == HOST)
        {
            session.play_automatic_action();
            session.reset_timer_for_current_turn();
        }
        let disconnected = session.turn_timer.as_ref().unwrap().player;
        let connection = connection_for_player(&session, disconnected);
        assert_ne!(connection, HOST);
        let records_before = session.game().unwrap().trick().unwrap().records().len();
        let deliveries = session.disconnect(connection);

        assert!(!deliveries.is_empty());
        let game = session.game().unwrap();
        assert!(matches!(
            game.trick().unwrap().records().get(records_before),
            Some(PlayRecord::Played { player, play })
                if *player == to_core_player(disconnected) && play.cards().len() == 1
        ));
        let timer = session.turn_timer_view().unwrap();
        assert_ne!(timer.player, disconnected);
        assert_eq!(timer.base_seconds, 5);
        assert_eq!(timer.reserve_seconds, 10);
    }

    #[test]
    fn only_ready_seated_room_host_can_start() {
        let mut session = QiGui523Session::new(ROOM, rules(), build_deck(1)).unwrap();
        join_three(&mut session);

        let non_host = session.handle(SECOND, message(3, ClientCommand::StartGame));
        assert_eq!(rejection(&non_host), Some(&RejectReason::OnlyHostCanStart));

        let not_ready = session.handle(HOST, message(3, ClientCommand::StartGame));
        assert!(matches!(
            rejection(&not_ready),
            Some(RejectReason::PlayersNotReady { .. })
        ));

        for connection in [HOST, SECOND, THIRD] {
            session.handle(
                connection,
                message(4, ClientCommand::SetReady { ready: true }),
            );
        }
        let deliveries = session.handle(HOST, message(5, ClientCommand::StartGame));
        assert!(session.game().is_some());
        assert_eq!(deliveries.len(), 3);
    }

    #[test]
    fn only_host_can_update_rules_and_changes_reset_ready_state() {
        let mut session = QiGui523Session::new(ROOM, rules(), build_deck(1)).unwrap();
        join_three(&mut session);
        for connection in [HOST, SECOND, THIRD] {
            session.handle(
                connection,
                message(3, ClientCommand::SetReady { ready: true }),
            );
        }
        let seats_before: HashMap<_, _> = session
            .players
            .iter()
            .map(|player| (player.connection, player.seat))
            .collect();
        let configured = RuleSet {
            deck_count: 2,
            hand_size: 10,
            suit_comparison: leocard_qigui523::SuitComparison::SumPoints,
            same_card_policy: leocard_qigui523::SameCardPolicy::CanFollow,
            ..rules()
        };

        let non_host = session.handle(
            SECOND,
            message(
                4,
                ClientCommand::Game(GameCommand::QiGui523(QiGui523Command::UpdateRules {
                    rules: configured,
                })),
            ),
        );
        assert_eq!(
            rejection(&non_host),
            Some(&RejectReason::OnlyHostCanConfigure)
        );
        let invalid = session.handle(
            HOST,
            message(
                4,
                ClientCommand::Game(GameCommand::QiGui523(QiGui523Command::UpdateRules {
                    rules: RuleSet {
                        deck_count: 0,
                        ..configured
                    },
                })),
            ),
        );
        assert_eq!(
            rejection(&invalid),
            Some(&RejectReason::InvalidRuleConfiguration)
        );

        let deliveries = session.handle(
            HOST,
            message(
                5,
                ClientCommand::Game(GameCommand::QiGui523(QiGui523Command::UpdateRules {
                    rules: configured,
                })),
            ),
        );
        assert_eq!(*session.rules(), configured);
        assert_eq!(session.shuffled_deck.as_ref().unwrap().len(), 108);
        assert!(
            session
                .players
                .iter()
                .all(|player| { player.ready == (player.connection == HOST) })
        );
        assert_eq!(
            session
                .players
                .iter()
                .map(|player| (player.connection, player.seat))
                .collect::<HashMap<_, _>>(),
            seats_before
        );
        assert_eq!(deliveries.len(), 3);
        assert!(deliveries.iter().all(|delivery| {
            matches!(
                &delivery.message.event,
                ServerEvent::LobbySnapshot(snapshot)
                    if snapshot.qigui523_rules() == Some(&configured)
                        && snapshot.players.iter().all(|player| {
                            player.ready == (Some(player.id) == snapshot.host)
                        })
            )
        }));
    }

    #[test]
    fn joining_assigns_unique_seats_and_the_host_is_always_ready() {
        let mut session = QiGui523Session::new(ROOM, rules(), build_deck(1)).unwrap();
        for (connection, name) in [(HOST, "房主"), (SECOND, "玩家二")] {
            session.handle(
                connection,
                message(1, join_command(name, ReconnectToken(connection.0))),
            );
        }

        let host_seat = session
            .players
            .iter()
            .find(|player| player.connection == HOST)
            .and_then(|player| player.seat)
            .unwrap();
        let second_seat = session
            .players
            .iter()
            .find(|player| player.connection == SECOND)
            .and_then(|player| player.seat)
            .unwrap();
        assert_ne!(host_seat, second_seat);
        assert!(
            session
                .players
                .iter()
                .find(|player| player.connection == HOST)
                .unwrap()
                .ready
        );
        assert!(
            !session
                .players
                .iter()
                .find(|player| player.connection == SECOND)
                .unwrap()
                .ready
        );

        session.handle(HOST, message(2, ClientCommand::SetReady { ready: false }));
        assert!(
            session
                .players
                .iter()
                .find(|player| player.connection == HOST)
                .unwrap()
                .ready
        );
        let occupied = session.handle(
            SECOND,
            message(2, ClientCommand::SelectSeat { seat: host_seat }),
        );
        assert_eq!(rejection(&occupied), Some(&RejectReason::SeatTaken));
        session.handle(SECOND, message(3, ClientCommand::SetReady { ready: true }));
        let deliveries = session.handle(HOST, message(4, ClientCommand::StartGame));

        assert_eq!(session.game().unwrap().rules().player_count, 2);
        assert_eq!(deliveries.len(), 2);
        let seats: Vec<_> = deliveries
            .iter()
            .filter_map(|delivery| match &delivery.message.event {
                ServerEvent::GameSnapshot(snapshot) => snapshot.qigui523().and_then(|snapshot| {
                    snapshot
                        .players
                        .iter()
                        .find(|player| player.id == snapshot.you)
                        .map(|player| player.seat)
                }),
                _ => None,
            })
            .collect();
        assert!(seats.contains(&host_seat));
        assert!(seats.contains(&second_seat));
    }

    #[cfg(feature = "developer")]
    #[test]
    fn developer_bot_seat_changes_keep_the_host_identity_stable() {
        let mut session = QiGui523Session::new(ROOM, rules(), build_deck(1)).unwrap();
        session.handle(
            HOST,
            message(1, join_command("房主", ReconnectToken(HOST.0))),
        );
        session.handle(
            HOST,
            message(2, ClientCommand::SelectSeat { seat: SeatId(5) }),
        );
        let rejected = session.handle(HOST, message(3, ClientCommand::StartGame));
        assert!(matches!(
            rejection(&rejected),
            Some(RejectReason::NotEnoughPlayers { .. })
        ));
        let first_bot_seat = SeatId(0);
        let second_bot_seat = SeatId(1);
        session.handle(
            HOST,
            message(
                4,
                ClientCommand::ConfigureBotSeat {
                    seat: first_bot_seat,
                    occupied: true,
                },
            ),
        );
        session.handle(
            HOST,
            message(
                5,
                ClientCommand::ConfigureBotSeat {
                    seat: second_bot_seat,
                    occupied: true,
                },
            ),
        );
        assert_eq!(session.player_id(HOST), Some(PlayerId(0)));
        assert_eq!(session.lobby_snapshot().host, Some(PlayerId(0)));

        session.handle(
            HOST,
            message(
                6,
                ClientCommand::ConfigureBotSeat {
                    seat: first_bot_seat,
                    occupied: false,
                },
            ),
        );
        assert_eq!(session.player_id(HOST), Some(PlayerId(0)));
        assert_eq!(
            session.players.iter().filter(|player| !player.left).count(),
            2
        );
        session.handle(
            HOST,
            message(
                7,
                ClientCommand::ConfigureBotSeat {
                    seat: second_bot_seat,
                    occupied: false,
                },
            ),
        );
        assert_eq!(
            session.players.iter().filter(|player| !player.left).count(),
            1
        );
        session.handle(
            HOST,
            message(
                8,
                ClientCommand::ConfigureBotSeat {
                    seat: first_bot_seat,
                    occupied: true,
                },
            ),
        );
        assert_eq!(session.player_id(HOST), Some(PlayerId(0)));
        let deliveries = session.handle(HOST, message(9, ClientCommand::StartGame));

        assert!(deliveries.iter().any(|delivery| matches!(
            delivery.message.event,
            ServerEvent::GameSnapshot(GameSnapshot::QiGui523(_))
        )));
        assert_eq!(session.players.len(), 2);
        let bot = session.players.iter().find(|player| player.is_bot).unwrap();
        assert_eq!(bot.profile_id, ProfileId([0; 32]));
        assert_eq!(bot.avatar, None);
        assert_eq!(bot.reference_points, 0);
        assert_eq!(bot.completed_games, 0);
        assert!(bot.auto_play);
        let snapshot = session.game_snapshot(session.room.host_player_id().unwrap());
        let bot_state = snapshot
            .players
            .iter()
            .find(|player| player.id == bot.id)
            .unwrap();
        assert!(bot_state.connected);
        assert!(bot_state.auto_play);
    }

    #[test]
    fn finished_game_returns_to_lobby_with_seats_preserved_and_ready_reset() {
        let mut session = QiGui523Session::new(ROOM, rules(), build_deck(1)).unwrap();
        for (connection, name, seat) in [
            (HOST, "房主", SeatId(4)),
            (SECOND, "玩家二", SeatId(1)),
            (THIRD, "玩家三", SeatId(5)),
        ] {
            session.handle(
                connection,
                message(1, join_command(name, ReconnectToken(connection.0))),
            );
            session.handle(connection, message(2, ClientCommand::SelectSeat { seat }));
            session.handle(
                connection,
                message(3, ClientCommand::SetReady { ready: true }),
            );
        }
        session.handle(HOST, message(4, ClientCommand::StartGame));

        let host_player = session.player_id(HOST).unwrap();
        assert_ne!(host_player, PlayerId(0));
        assert_eq!(session.game_snapshot(host_player).host, host_player);
        let during_game = session.handle(HOST, message(5, ClientCommand::ReturnToLobby));
        assert_eq!(
            rejection(&during_game),
            Some(&RejectReason::GameNotFinished)
        );

        finish_game(&mut session);
        let non_host = session.handle(SECOND, message(5, ClientCommand::ReturnToLobby));
        assert_eq!(
            rejection(&non_host),
            Some(&RejectReason::OnlyHostCanReturnToLobby)
        );
        let seats_before: HashMap<_, _> = session
            .players
            .iter()
            .map(|player| (player.connection, player.seat))
            .collect();
        let deliveries = session.handle(HOST, message(6, ClientCommand::ReturnToLobby));

        assert!(session.game().is_none());
        assert!(session.shuffled_deck.is_some());
        assert!(
            session
                .players
                .iter()
                .all(|player| { player.ready == (player.connection == HOST) })
        );
        assert_eq!(
            session
                .players
                .iter()
                .map(|player| (player.connection, player.seat))
                .collect::<HashMap<_, _>>(),
            seats_before
        );
        assert_eq!(deliveries.len(), 3);
        for delivery in deliveries {
            let ServerEvent::LobbySnapshot(snapshot) = delivery.message.event else {
                panic!("returning to the lobby broadcasts lobby snapshots");
            };
            assert_eq!(snapshot.host, Some(host_player));
            assert!(
                snapshot
                    .players
                    .iter()
                    .all(|player| { player.ready == (Some(player.id) == snapshot.host) })
            );
        }

        for connection in [HOST, SECOND, THIRD] {
            session.handle(
                connection,
                message(7, ClientCommand::SetReady { ready: true }),
            );
        }
        let second_game = session.handle(HOST, message(8, ClientCommand::StartGame));
        assert!(session.game().is_some());
        assert_eq!(second_game.len(), 3);
    }

    #[test]
    fn returning_to_lobby_excludes_players_who_disconnected_after_game_end() {
        let mut session = QiGui523Session::new(ROOM, rules(), build_deck(1)).unwrap();
        join_three(&mut session);
        ready_and_start(&mut session);
        finish_game(&mut session);
        session.disconnect(SECOND);

        let deliveries = send_next(&mut session, HOST, ClientCommand::ReturnToLobby);

        assert_eq!(deliveries.len(), 2);
        assert!(deliveries.iter().all(|delivery| {
            matches!(
                &delivery.message.event,
                ServerEvent::LobbySnapshot(snapshot)
                    if snapshot.players.iter().all(|player| player.name != "玩家二")
            )
        }));
    }

    #[test]
    fn finished_players_ready_in_place_and_the_last_one_starts_the_next_game() {
        let mut session = QiGui523Session::new(ROOM, rules(), build_deck(1)).unwrap();
        join_three(&mut session);
        ready_and_start(&mut session);
        finish_game(&mut session);
        assert!(session.players.iter().all(|player| !player.ready));

        for connection in [HOST, SECOND] {
            let deliveries = send_next(&mut session, connection, ClientCommand::PlayAgain);
            assert!(matches!(
                session.game().unwrap().phase(),
                Phase::Finished(_)
            ));
            assert!(deliveries.iter().all(|delivery| {
                qigui523_snapshot(&delivery.message.event)
                    .is_some_and(|snapshot| snapshot.players.iter().any(|player| player.ready))
            }));
        }

        let deliveries = send_next(&mut session, THIRD, ClientCommand::PlayAgain);
        assert!(matches!(session.game().unwrap().phase(), Phase::Playing));
        assert_eq!(session.game().unwrap().players().len(), 3);
        assert_eq!(deliveries.len(), 3);
        assert!(deliveries.iter().all(|delivery| {
            qigui523_snapshot(&delivery.message.event).is_some_and(|snapshot| {
                matches!(snapshot.phase, GamePhaseView::Playing)
                    && snapshot.players.iter().all(|player| !player.ready)
            })
        }));
    }

    #[test]
    fn player_still_in_auto_play_is_ready_and_remains_in_auto_play_next_game() {
        let mut session = QiGui523Session::new(ROOM, rules(), build_deck(1)).unwrap();
        join_three(&mut session);
        ready_and_start(&mut session);
        session
            .players
            .iter_mut()
            .find(|player| player.connection == SECOND)
            .unwrap()
            .auto_play = true;

        finish_game(&mut session);
        session.broadcast_game_after_update(None);

        let auto_player = session
            .players
            .iter()
            .find(|player| player.connection == SECOND)
            .unwrap();
        assert!(auto_player.ready);
        assert!(auto_player.auto_play);
        assert!(
            session
                .players
                .iter()
                .filter(|player| player.connection != SECOND)
                .all(|player| !player.ready)
        );

        send_next(&mut session, HOST, ClientCommand::PlayAgain);
        send_next(&mut session, THIRD, ClientCommand::PlayAgain);

        assert!(matches!(session.game().unwrap().phase(), Phase::Playing));
        let auto_player = session
            .players
            .iter()
            .find(|player| player.connection == SECOND)
            .unwrap();
        assert!(!auto_player.ready);
        assert!(auto_player.auto_play);
    }

    #[test]
    fn guest_leaves_with_a_named_notice_and_no_longer_blocks_the_next_game() {
        let mut session = QiGui523Session::new(ROOM, rules(), build_deck(1)).unwrap();
        join_three(&mut session);
        ready_and_start(&mut session);
        finish_game(&mut session);

        let deliveries = send_next(&mut session, SECOND, ClientCommand::LeaveRoom);
        assert!(deliveries.iter().any(|delivery| {
            delivery.recipient == SECOND && matches!(delivery.message.event, ServerEvent::LeftRoom)
        }));
        assert_eq!(
            deliveries
                .iter()
                .filter(|delivery| matches!(
                    &delivery.message.event,
                    ServerEvent::PlayerLeft { name } if name == "玩家二"
                ))
                .count(),
            2
        );

        send_next(&mut session, HOST, ClientCommand::PlayAgain);
        let next_game = send_next(&mut session, THIRD, ClientCommand::PlayAgain);
        assert!(matches!(session.game().unwrap().phase(), Phase::Playing));
        assert_eq!(session.game().unwrap().players().len(), 2);
        assert_eq!(next_game.len(), 2);
        assert!(session.players.iter().all(|player| player.name != "玩家二"));
    }

    #[test]
    fn player_still_offline_at_game_end_is_automatically_departed() {
        let mut session = QiGui523Session::new(ROOM, rules(), build_deck(1)).unwrap();
        join_three(&mut session);
        ready_and_start(&mut session);
        session.disconnect(SECOND);
        assert!(
            !session
                .players
                .iter()
                .find(|player| player.connection == SECOND)
                .unwrap()
                .left
        );

        finish_game(&mut session);
        let deliveries = session.broadcast_game_after_update(None);

        let departed = session
            .players
            .iter()
            .find(|player| player.connection == SECOND)
            .unwrap();
        assert!(departed.left);
        assert!(!departed.ready);
        assert_eq!(
            deliveries
                .iter()
                .filter(|delivery| matches!(
                    &delivery.message.event,
                    ServerEvent::PlayerLeft { name } if name == "玩家二"
                ))
                .count(),
            2
        );

        send_next(&mut session, HOST, ClientCommand::PlayAgain);
        send_next(&mut session, THIRD, ClientCommand::PlayAgain);
        assert_eq!(session.game().unwrap().players().len(), 2);
        assert!(matches!(session.game().unwrap().phase(), Phase::Playing));
    }

    #[test]
    fn host_disconnect_closes_the_room_immediately() {
        let mut session = QiGui523Session::new(ROOM, rules(), build_deck(1)).unwrap();
        join_three(&mut session);
        ready_and_start(&mut session);

        let deliveries = session.disconnect(HOST);

        assert!(session.is_closed());
        assert!(matches!(session.game().unwrap().phase(), Phase::Playing));
        assert_eq!(deliveries.len(), 2);
        assert!(
            deliveries
                .iter()
                .all(|delivery| matches!(delivery.message.event, ServerEvent::RoomClosed))
        );
    }

    #[test]
    fn host_leave_command_during_a_game_closes_the_room_for_everyone() {
        let mut session = QiGui523Session::new(ROOM, rules(), build_deck(1)).unwrap();
        join_three(&mut session);
        ready_and_start(&mut session);

        let deliveries = send_next(&mut session, HOST, ClientCommand::LeaveRoom);

        assert!(session.is_closed());
        assert_eq!(deliveries.len(), 3);
        assert!(
            deliveries
                .iter()
                .all(|delivery| matches!(delivery.message.event, ServerEvent::RoomClosed))
        );
    }

    #[test]
    fn lobby_leave_slot_is_reused_without_renumbering_the_remaining_players() {
        let mut session = QiGui523Session::new(ROOM, rules(), build_deck(1)).unwrap();
        join_three(&mut session);
        send_next(&mut session, SECOND, ClientCommand::LeaveRoom);

        let fourth = ConnectionId(40);
        let deliveries = session.handle(
            fourth,
            message(1, join_command("新玩家", ReconnectToken(fourth.0))),
        );

        assert_eq!(session.player_id(THIRD), Some(PlayerId(2)));
        assert_eq!(session.player_id(fourth), Some(PlayerId(1)));
        assert!(deliveries.iter().any(|delivery| matches!(
            &delivery.message.event,
            ServerEvent::Joined { you: PlayerId(1) }
        )));
        assert!(
            session
                .lobby_snapshot()
                .players
                .iter()
                .all(|player| player.name != "玩家二")
        );
    }

    #[test]
    fn seventh_connection_is_rejected_from_the_six_player_room() {
        let mut session = QiGui523Session::new(ROOM, rules(), build_deck(1)).unwrap();
        for index in 0..6_u64 {
            session.handle(
                ConnectionId(100 + index),
                message(
                    1,
                    join_command(&format!("P{index}"), ReconnectToken(100 + index)),
                ),
            );
        }
        let seventh = session.handle(
            ConnectionId(200),
            message(1, join_command("P6", ReconnectToken(200))),
        );
        assert_eq!(rejection(&seventh), Some(&RejectReason::RoomFull));
    }

    #[test]
    fn normalized_avatar_is_sent_once_to_each_current_or_late_joiner() {
        let mut session = QiGui523Session::new(ROOM, rules(), build_deck(1)).unwrap();
        session.handle(
            HOST,
            message(1, join_command("房主", ReconnectToken(HOST.0))),
        );
        let png = avatar_png(AVATAR_DIMENSION, AVATAR_DIMENSION);
        let upload = session.handle(
            HOST,
            message(2, ClientCommand::SetAvatar { png: png.clone() }),
        );
        assert_eq!(
            upload
                .iter()
                .filter(|delivery| {
                    matches!(&delivery.message.event, ServerEvent::AvatarData { .. })
                })
                .count(),
            1
        );

        let late_join = session.handle(
            SECOND,
            message(1, join_command("玩家二", ReconnectToken(SECOND.0))),
        );
        let avatars: Vec<_> = late_join
            .iter()
            .filter(|delivery| delivery.recipient == SECOND)
            .filter_map(|delivery| match &delivery.message.event {
                ServerEvent::AvatarData { id, png } => Some((*id, png)),
                _ => None,
            })
            .collect();
        assert_eq!(avatars.len(), 1);
        assert_eq!(avatars[0].1, &png);

        let duplicate = session.handle(
            HOST,
            message(3, ClientCommand::SetAvatar { png: png.clone() }),
        );
        assert_eq!(rejection(&duplicate), Some(&RejectReason::AvatarAlreadySet));
        let invalid = session.handle(
            SECOND,
            message(
                2,
                ClientCommand::SetAvatar {
                    png: avatar_png(AVATAR_DIMENSION / 2, AVATAR_DIMENSION / 2),
                },
            ),
        );
        assert_eq!(rejection(&invalid), Some(&RejectReason::InvalidAvatar));
    }

    #[test]
    fn each_game_snapshot_contains_only_its_recipient_hand() {
        let mut session = QiGui523Session::new(ROOM, rules(), build_deck(1)).unwrap();
        join_three(&mut session);
        let deliveries = ready_and_start(&mut session);
        let game = session.game().unwrap();

        for delivery in deliveries {
            let ServerEvent::GameSnapshot(snapshot) = delivery.message.event else {
                panic!("start broadcasts personalized game snapshots");
            };
            let snapshot = snapshot.into_qigui523().expect("七鬼五二三快照");
            let expected_player = session.player_id(delivery.recipient).unwrap();
            assert_eq!(snapshot.you, expected_player);
            assert_eq!(
                snapshot.your_hand,
                game.players()[usize::from(expected_player.0)].hand()
            );
            assert!(
                snapshot
                    .players
                    .iter()
                    .all(|player| player.hand_len == rules().hand_size.into())
            );
        }
    }

    #[test]
    fn duplicate_action_request_never_executes_twice() {
        let mut session = QiGui523Session::new(ROOM, rules(), build_deck(1)).unwrap();
        join_three(&mut session);
        ready_and_start(&mut session);

        let current = session.game().unwrap().trick().unwrap().current_player();
        let connection = session.players[current.0].connection;
        let card = session.game().unwrap().players()[current.0].hand()[0];
        let command = message(
            10,
            ClientCommand::Game(GameCommand::QiGui523(QiGui523Command::PlayCards {
                cards: vec![card],
            })),
        );
        let revision_before = session.revision();

        session.handle(connection, command.clone());
        let revision_after_first = session.revision();
        let hand_after_first = session.game().unwrap().players()[current.0].hand().len();
        let duplicate = session.handle(connection, command);

        assert_eq!(revision_after_first.0, revision_before.0 + 1);
        assert_eq!(session.revision(), revision_after_first);
        assert_eq!(
            session.game().unwrap().players()[current.0].hand().len(),
            hand_after_first
        );
        assert_eq!(
            rejection(&duplicate),
            Some(&RejectReason::DuplicateRequest {
                last_seen: RequestId(10)
            })
        );
    }

    #[test]
    fn accepted_play_broadcasts_one_effect_event_to_every_connected_player() {
        let mut session = QiGui523Session::new(ROOM, rules(), build_deck(1)).unwrap();
        join_three(&mut session);
        ready_and_start(&mut session);
        let starting = session.game().unwrap().starting_card();
        let player = from_core_player(starting.player);
        let connection = connection_for_player(&session, player);

        let deliveries = send_next(
            &mut session,
            connection,
            ClientCommand::Game(GameCommand::QiGui523(QiGui523Command::PlayCards {
                cards: vec![starting.card],
            })),
        );

        assert_eq!(
            deliveries
                .iter()
                .filter(|delivery| matches!(
                    &delivery.message.event,
                    ServerEvent::GameEvent(GameEvent::QiGui523(
                        QiGui523Event::PlayEffect {
                            player: effect_player,
                            play,
                        }
                    )) if *effect_player == player
                        && play.cards == vec![starting.card]
                        && matches!(play.kind, leocard_qigui523::PlayKind::Single)
                ))
                .count(),
            3
        );
    }

    #[test]
    fn player_interaction_is_broadcast_identically_to_every_connected_player() {
        let mut session = QiGui523Session::new(ROOM, rules(), build_deck(1)).unwrap();
        join_three(&mut session);
        ready_and_start(&mut session);

        let deliveries = session.handle(
            HOST,
            message(
                10,
                ClientCommand::Interact {
                    target: PlayerId(2),
                    kind: PlayerInteractionKind::Egg,
                },
            ),
        );
        let interactions = deliveries
            .iter()
            .filter_map(|delivery| match delivery.message.event {
                ServerEvent::PlayerInteraction(interaction) => Some((delivery, interaction)),
                _ => None,
            })
            .collect::<Vec<_>>();

        assert_eq!(interactions.len(), 3);
        assert!(interactions.iter().all(|(_, interaction)| {
            interaction.source == PlayerId(0)
                && interaction.target == PlayerId(2)
                && interaction.kind == PlayerInteractionKind::Egg
                && interaction.seed == interactions[0].1.seed
        }));
        assert!(interactions.iter().all(|(delivery, _)| {
            delivery.message.in_reply_to == (delivery.recipient == HOST).then_some(RequestId(10))
        }));
        assert_eq!(
            session
                .players
                .iter()
                .find(|player| player.id == PlayerId(2))
                .and_then(|player| player.game_profiles.interactions.as_ref())
                .map(|stats| stats.eggs_received),
            Some(1)
        );
    }

    #[test]
    fn interaction_profile_converts_wine_and_shoe_into_ten_items() {
        let mut session = QiGui523Session::new(ROOM, rules(), build_deck(1)).unwrap();
        join_three(&mut session);

        session.record_received_interaction(PlayerId(1), PlayerInteractionKind::Flower);
        session.record_received_interaction(PlayerId(1), PlayerInteractionKind::Wine);
        session.record_received_interaction(PlayerId(1), PlayerInteractionKind::Egg);
        session.record_received_interaction(PlayerId(1), PlayerInteractionKind::Shoe);

        let stats = session
            .players
            .iter()
            .find(|player| player.id == PlayerId(1))
            .and_then(|player| player.game_profiles.interactions.as_ref())
            .unwrap();
        assert_eq!(stats.flowers_received, 11);
        assert_eq!(stats.eggs_received, 11);
    }

    #[test]
    fn chat_is_validated_and_broadcast_to_every_connected_player() {
        let mut session = QiGui523Session::new(ROOM, rules(), build_deck(1)).unwrap();
        join_three(&mut session);
        ready_and_start(&mut session);

        let deliveries = session.handle(
            HOST,
            message(
                10,
                ClientCommand::Chat {
                    content: ChatContent::Text("  大家好  ".to_owned()),
                },
            ),
        );
        let chats = deliveries
            .iter()
            .filter_map(|delivery| match &delivery.message.event {
                ServerEvent::ChatMessage(chat) => Some((delivery, chat)),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(chats.len(), 3);
        assert!(chats.iter().all(|(_, chat)| {
            chat.source == PlayerId(0) && chat.content == ChatContent::Text("大家好".to_owned())
        }));
        assert!(chats.iter().all(|(delivery, _)| {
            delivery.message.in_reply_to == (delivery.recipient == HOST).then_some(RequestId(10))
        }));

        let invalid = session.handle(
            HOST,
            message(
                11,
                ClientCommand::Chat {
                    content: ChatContent::QuickVoice(QUICK_VOICE_COUNT),
                },
            ),
        );
        assert_eq!(rejection(&invalid), Some(&RejectReason::InvalidChatMessage));
    }

    #[test]
    fn a_player_cannot_send_an_interaction_to_themselves() {
        let mut session = QiGui523Session::new(ROOM, rules(), build_deck(1)).unwrap();
        join_three(&mut session);
        ready_and_start(&mut session);

        let deliveries = session.handle(
            SECOND,
            message(
                10,
                ClientCommand::Interact {
                    target: PlayerId(1),
                    kind: PlayerInteractionKind::Flower,
                },
            ),
        );

        assert_eq!(
            rejection(&deliveries),
            Some(&RejectReason::GameViolation(GameViolation::QiGui523(
                RuleViolation::InvalidPlayer,
            )))
        );
    }

    #[test]
    fn rejected_rule_action_does_not_mutate_authoritative_state() {
        let mut session = QiGui523Session::new(ROOM, rules(), build_deck(1)).unwrap();
        join_three(&mut session);
        ready_and_start(&mut session);

        let current = session.game().unwrap().trick().unwrap().current_player();
        let wrong_index = (current.0 + 1) % 3;
        let wrong_connection = session.players[wrong_index].connection;
        let before = session.game().unwrap().clone();
        let revision = session.revision();
        let deliveries = session.handle(
            wrong_connection,
            message(
                10,
                ClientCommand::Game(GameCommand::QiGui523(QiGui523Command::Pass)),
            ),
        );

        assert_eq!(session.game(), Some(&before));
        assert_eq!(session.revision(), revision);
        assert_eq!(
            rejection(&deliveries),
            Some(&RejectReason::GameViolation(GameViolation::QiGui523(
                RuleViolation::NotPlayersTurn,
            )))
        );
    }

    #[test]
    fn protocol_and_room_mismatch_never_touch_room_state() {
        let mut session = QiGui523Session::new(ROOM, rules(), build_deck(1)).unwrap();
        let mut wrong_protocol = message(1, join_command("玩家", ReconnectToken(HOST.0)));
        wrong_protocol.protocol_version += 1;
        let protocol_reply = session.handle(HOST, wrong_protocol);

        let mut wrong_room = message(1, join_command("玩家", ReconnectToken(HOST.0)));
        wrong_room.room_id = RoomId(999);
        let room_reply = session.handle(HOST, wrong_room);

        assert!(matches!(
            rejection(&protocol_reply),
            Some(RejectReason::ProtocolMismatch { .. })
        ));
        assert_eq!(rejection(&room_reply), Some(&RejectReason::RoomMismatch));
        assert!(session.players.is_empty());
        assert_eq!(session.revision(), Revision(0));
    }

    #[test]
    fn disconnect_is_broadcast_once_only_to_connected_players() {
        let mut session = QiGui523Session::new(ROOM, rules(), build_deck(1)).unwrap();
        join_three(&mut session);
        let revision = session.revision();

        let deliveries = session.disconnect(SECOND);

        assert_eq!(session.revision().0, revision.0 + 1);
        assert_eq!(deliveries.len(), 2);
        assert!(
            deliveries
                .iter()
                .all(|delivery| delivery.recipient != SECOND)
        );
        for delivery in deliveries {
            let ServerEvent::LobbySnapshot(snapshot) = delivery.message.event else {
                panic!("disconnect before a game broadcasts lobby snapshots");
            };
            assert_eq!(snapshot.players.len(), 2);
            assert!(snapshot.players.iter().all(|player| player.connected));
            assert!(
                snapshot
                    .players
                    .iter()
                    .all(|player| player.id != PlayerId(1))
            );
        }

        assert!(session.players[usize::from(PlayerId(1).0)].left);

        let freed_seat = session.handle(
            HOST,
            message(4, ClientCommand::SelectSeat { seat: SeatId(1) }),
        );
        assert_eq!(rejection(&freed_seat), None);
        assert_eq!(
            session
                .players
                .iter()
                .find(|player| player.connection == HOST)
                .unwrap()
                .seat,
            Some(SeatId(1))
        );

        let revision_after_reseat = session.revision();
        assert!(session.disconnect(SECOND).is_empty());
        assert_eq!(session.revision(), revision_after_reseat);

        let replacement = ConnectionId(99);
        let rejoined = session.handle(
            replacement,
            message(1, join_command("玩家二", ReconnectToken(SECOND.0))),
        );
        assert_eq!(session.player_id(replacement), Some(PlayerId(1)));
        assert!(!session.players[usize::from(PlayerId(1).0)].left);
        assert!(rejoined.iter().any(|delivery| matches!(
            &delivery.message.event,
            ServerEvent::LobbySnapshot(snapshot) if snapshot.players.len() == 3
        )));
    }

    #[test]
    fn reconnect_during_game_restores_the_same_player_and_private_hand() {
        let mut session = QiGui523Session::new(ROOM, rules(), build_deck(1)).unwrap();
        join_three(&mut session);
        ready_and_start(&mut session);
        let player = session.player_id(SECOND).unwrap();
        let hand_before = session.game_snapshot(player).your_hand;

        session.disconnect(SECOND);
        let replacement = ConnectionId(99);
        let deliveries = session.handle(
            replacement,
            message(1, join_command("玩家二", ReconnectToken(SECOND.0))),
        );

        assert_eq!(session.player_id(SECOND), None);
        assert_eq!(session.player_id(replacement), Some(player));
        assert!(session.players[usize::from(player.0)].connected);
        assert!(deliveries.iter().any(|delivery| {
            delivery.recipient == replacement
                && matches!(
                    delivery.message.event,
                    ServerEvent::Joined { you } if you == player
                )
        }));
        let restored_hand = deliveries.iter().find_map(|delivery| {
            if delivery.recipient != replacement {
                return None;
            }
            let ServerEvent::GameSnapshot(snapshot) = &delivery.message.event else {
                return None;
            };
            Some(snapshot.qigui523()?.your_hand.clone())
        });
        assert_eq!(restored_hand, Some(hand_before));
    }
}
