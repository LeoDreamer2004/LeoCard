use super::{QiGui523Session, validate_deck};
use crate::lifecycle::{HostedGameLifecycle, dispatch_client_command};
use crate::{
    AUTO_PLAY_DELAY, AutoPlayDelayState, ConnectionId, Delivery, HostError, RoomSession,
    new_match_id,
};
use leocard_protocol::{
    ClientMessage, GameCommand, GameKind, GameRules, GameSnapshot, GameViolation, PlayerId,
    PlayerInteraction, PlayerInteractionKind, PlayerViolation, QiGui523Command,
    QiGui523ProfileStats, RejectReason, RequestId, Revision, RoomId, RoomViolation, RuleViolation,
    ServerEvent,
};
use leocard_qigui523::{GameState, Phase, QiGuiCard, QiGuiRuleSet, build_deck};
use std::time::Duration;

impl QiGui523Session {
    /// `shuffled_deck[0]` 是第一张发出的牌；房主应在创建会话前完成洗牌。
    pub fn new(
        room_id: RoomId,
        rules: QiGuiRuleSet,
        shuffled_deck: Vec<QiGuiCard>,
    ) -> Result<Self, HostError> {
        Self::new_with_host_port(room_id, 52300, rules, shuffled_deck)
    }

    pub fn new_with_host_port(
        room_id: RoomId,
        host_port: u16,
        rules: QiGuiRuleSet,
        shuffled_deck: Vec<QiGuiCard>,
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

    pub fn rules(&self) -> &QiGuiRuleSet {
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

    pub fn is_current_connection(&self, connection: ConnectionId) -> bool {
        self.player_id(connection).is_some()
    }

    /// 生成不改变权威修订号的轻量心跳，只发给当前在线玩家。
    pub fn heartbeat(&self) -> Vec<Delivery> {
        self.room.heartbeat()
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
            return self.room.broadcast_event(None, ServerEvent::RoomClosed);
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
        dispatch_client_command(self, connection, message)
    }
}

impl HostedGameLifecycle for QiGui523Session {
    const KIND: GameKind = GameKind::QiGui523;

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
        self.rules.player_count
    }

    fn game_rules(&self) -> GameRules {
        self.rules.into()
    }

    fn game_snapshot(&self, recipient: PlayerId) -> GameSnapshot {
        QiGui523Session::game_snapshot(self, recipient).into()
    }

    fn handle_game_command(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        command: GameCommand,
    ) -> Result<Vec<Delivery>, GameKind> {
        let GameCommand::QiGui523(command) = command else {
            return Err(command.kind());
        };
        Ok(match command {
            QiGui523Command::SetAutoPlay { enabled } => {
                self.set_auto_play(connection, request_id, enabled)
            }
            QiGui523Command::UpdateRules { rules } => {
                self.update_rules(connection, request_id, rules)
            }
            QiGui523Command::PlayCards { cards } => self.play_cards(connection, request_id, &cards),
            QiGui523Command::SetDeveloperHand { cards } => {
                self.set_developer_hand(connection, request_id, cards)
            }
            QiGui523Command::Pass => self.pass(connection, request_id),
        })
    }

    fn start_game(&mut self, connection: ConnectionId, request_id: RequestId) -> Vec<Delivery> {
        if self.player_id(connection).is_none() {
            return self.reject(
                connection,
                request_id,
                RejectReason::Player(PlayerViolation::NotJoined),
            );
        }
        if self.game.is_some() {
            return self.reject(
                connection,
                request_id,
                RejectReason::Game(GameViolation::GameAlreadyStarted),
            );
        }
        if self.host_connection != Some(connection) {
            return self.reject(
                connection,
                request_id,
                RejectReason::Room(RoomViolation::OnlyHostCanStart),
            );
        }
        let active_player_count = self.players.iter().filter(|player| !player.left).count();
        if active_player_count < 2 {
            return self.reject(
                connection,
                request_id,
                RejectReason::Room(RoomViolation::NotEnoughPlayers {
                    minimum: 2,
                    actual: active_player_count as u8,
                }),
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
            return self.reject(
                connection,
                request_id,
                RejectReason::Room(RoomViolation::MustSelectSeat),
            );
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
                RejectReason::Room(RoomViolation::PlayersNotReady { players: not_ready }),
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
        let game_rules = QiGuiRuleSet {
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
            return self.reject(
                connection,
                request_id,
                RejectReason::Player(PlayerViolation::NotJoined),
            );
        }
        if self.host_connection != Some(connection) {
            return self.reject(
                connection,
                request_id,
                RejectReason::Room(RoomViolation::OnlyHostCanReturnToLobby),
            );
        }
        let Some(game) = self.game.as_ref() else {
            return self.reject(
                connection,
                request_id,
                RejectReason::Game(GameViolation::GameNotStarted),
            );
        };
        if !matches!(game.phase(), Phase::Finished(_)) {
            return self.reject(
                connection,
                request_id,
                RejectReason::Game(GameViolation::GameNotFinished),
            );
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
            return self.reject(
                connection,
                request_id,
                RejectReason::Player(PlayerViolation::NotJoined),
            );
        };
        let Some(game) = self.game.as_ref() else {
            return self.reject(
                connection,
                request_id,
                RejectReason::Game(GameViolation::GameNotStarted),
            );
        };
        if !matches!(game.phase(), Phase::Finished(_)) {
            return self.reject(
                connection,
                request_id,
                RejectReason::Game(GameViolation::GameNotFinished),
            );
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
            return self.reject(
                connection,
                request_id,
                RejectReason::Player(PlayerViolation::NotJoined),
            );
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
            self.room
                .broadcast_event(None, ServerEvent::PlayerLeft { name }),
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

    fn interact(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        target: PlayerId,
        kind: PlayerInteractionKind,
    ) -> Vec<Delivery> {
        let Some(source) = self.player_id(connection) else {
            return self.reject(
                connection,
                request_id,
                RejectReason::Player(PlayerViolation::NotJoined),
            );
        };
        let Some(game) = self.game.as_ref() else {
            return self.reject(
                connection,
                request_id,
                RejectReason::Game(GameViolation::GameNotStarted),
            );
        };
        if !matches!(game.phase(), Phase::Playing) {
            return self.reject(
                connection,
                request_id,
                RejectReason::Game(GameViolation::QiGui523(RuleViolation::GameAlreadyFinished)),
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
                RejectReason::Game(GameViolation::QiGui523(RuleViolation::InvalidPlayer)),
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
        let mut deliveries = self.room.broadcast_event(
            Some((connection, request_id)),
            ServerEvent::PlayerInteraction(interaction),
        );
        deliveries.extend(self.broadcast_game(None));
        deliveries
    }

    fn before_dispatch(&mut self) {
        self.apply_finished_reference_points();
    }
}
