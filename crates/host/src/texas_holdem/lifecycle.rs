use super::{TexasHoldemAdapter, TexasHoldemSession, validate_deck};
use crate::lifecycle::{HostedGameLifecycle, dispatch_client_command};
use crate::{AutomaticActionDelayState, ConnectionId, Delivery, HostError, RoomSession};
use leocard_protocol::{
    ClientMessage, GameCommand, GameKind, GameRules, GameSnapshot, GameViolation, PlayerViolation,
    RejectReason, RequestId, Revision, RoomId, RoomViolation, ServerEvent, TexasHoldemCommand,
};
use leocard_texas_holdem::{Phase, TexasHoldemCard, TexasHoldemRuleSet, build_deck};
use std::time::Duration;

impl TexasHoldemSession {
    pub fn new(
        room_id: RoomId,
        host_port: u16,
        rules: TexasHoldemRuleSet,
        shuffled_deck: Vec<TexasHoldemCard>,
    ) -> Result<Self, HostError> {
        let rules = TexasHoldemRuleSet {
            player_count: TexasHoldemRuleSet::MAX_PLAYERS,
            ..rules
        }
        .validate_options()?;
        validate_deck(rules.short_deck, &shuffled_deck)?;
        Ok(Self {
            room: RoomSession::new_with_seat_count(
                room_id,
                host_port,
                TexasHoldemRuleSet::MAX_PLAYERS,
                TexasHoldemRuleSet::MAX_PLAYERS,
            ),
            rules,
            shuffled_deck: Some(shuffled_deck),
            game: None,
            match_profile_stats: Vec::new(),
            finished_reference_changes: None,
            automatic_action_delay: None,
            bot_decision: Default::default(),
        })
    }

    pub const fn room_id(&self) -> RoomId {
        self.room.room_id
    }

    pub const fn rules(&self) -> &TexasHoldemRuleSet {
        &self.rules
    }

    pub const fn revision(&self) -> Revision {
        self.room.revision
    }

    pub const fn game(&self) -> Option<&TexasHoldemAdapter> {
        self.game.as_ref()
    }

    pub const fn is_closed(&self) -> bool {
        self.room.closed
    }

    pub fn is_current_connection(&self, connection: ConnectionId) -> bool {
        self.room.player_id(connection).is_some()
    }

    pub fn heartbeat(&self) -> Vec<Delivery> {
        self.room.heartbeat()
    }

    /// 推进自动盲注及托管机器人的行动延迟。
    pub fn advance_time(&mut self, elapsed: Duration) -> Vec<Delivery> {
        let mut deliveries = self.advance_automatic_turn(elapsed);
        if self
            .game
            .as_ref()
            .is_some_and(TexasHoldemAdapter::poll_spectator_equities)
        {
            self.room.bump_revision();
            deliveries.extend(self.broadcast_game(None));
        }
        deliveries
    }

    fn advance_automatic_turn(&mut self, elapsed: Duration) -> Vec<Delivery> {
        if elapsed.is_zero() || self.game.is_none() {
            return Vec::new();
        }
        let Some(player) = self.current_automatic_action_player() else {
            self.automatic_action_delay = None;
            self.bot_decision.clear();
            return Vec::new();
        };
        let remaining = self.automatic_delay_for_current_turn();
        let delay = self
            .automatic_action_delay
            .get_or_insert(AutomaticActionDelayState { player, remaining });
        if delay.player != player {
            *delay = AutomaticActionDelayState { player, remaining };
        }
        if elapsed < delay.remaining {
            delay.remaining -= elapsed;
            return Vec::new();
        }
        delay.remaining = Duration::ZERO;
        let Some(events) = self.play_automatic_action() else {
            return Vec::new();
        };
        self.room.bump_revision();
        let mut deliveries = self.broadcast_events(events);
        deliveries.extend(self.broadcast_game(None));
        deliveries
    }

    pub fn disconnect(&mut self, connection: ConnectionId) -> Vec<Delivery> {
        if self.room.disconnect_spectator(connection) {
            return Vec::new();
        }
        let Some(index) =
            self.room.players.iter().position(|player| {
                player.connection == connection && player.connected && !player.left
            })
        else {
            return Vec::new();
        };
        if self.room.host_connection == Some(connection) {
            self.room.players[index].connected = false;
            self.room.bump_revision();
            self.room.closed = true;
            return self.room.broadcast_event(None, ServerEvent::RoomClosed);
        }

        let player = self.room.players[index].id;
        self.room.players[index].connected = false;
        if let Some(game) = self.game.as_mut() {
            let _ = game.set_connected(player, false);
        } else {
            self.room.players[index].seat = None;
            self.room.players[index].ready = false;
            // 大厅断线无需保留牌局状态；释放参与者槽位，避免离线记录继续占用
            // 人数和房间容量。牌局中的断线玩家仍保留，以支持恢复底牌和筹码。
            self.room.players[index].left = true;
        }
        let was_complete = self
            .game
            .as_ref()
            .is_some_and(|game| matches!(game.game().phase(), Phase::Complete(_)));
        let events = self.fold_disconnected_players();
        self.record_profile_events(&events);
        self.finish_hand_if_needed(was_complete);
        self.reset_automatic_action_delay_for_current_turn();
        self.room.bump_revision();
        if self.game.is_some() {
            let mut deliveries = self.broadcast_events(events);
            deliveries.extend(self.broadcast_game(None));
            deliveries
        } else {
            self.broadcast_lobby(None)
        }
    }

    pub fn handle(&mut self, connection: ConnectionId, message: ClientMessage) -> Vec<Delivery> {
        let request_id = message.request_id;
        let mut deliveries = dispatch_client_command(self, connection, message);
        if !self.room.closed
            && self.game.is_none()
            && self.rules.short_deck
            && self.rules.omaha
            && self
                .room
                .players
                .iter()
                .filter(|player| !player.left)
                .count()
                > 6
        {
            self.rules.short_deck = false;
            self.room.reset_ready_after_rules_change();
            let mut deck = build_deck(false);
            fastrand::shuffle(&mut deck);
            self.shuffled_deck = Some(deck);
            self.room.bump_revision();
            // Replace snapshots produced before the adjustment, so no peer sees invalid rules.
            deliveries.retain(|delivery| {
                !matches!(delivery.message.event, ServerEvent::LobbySnapshot(_))
            });
            deliveries.extend(self.broadcast_lobby(Some((connection, request_id))));
            if let Some(host) = self.room.host_connection {
                deliveries.push(self.room.delivery(
                    host,
                    None,
                    ServerEvent::Notice {
                        title: "规则已调整".to_owned(),
                        message: "短牌奥马哈至多六人，已自动关闭短牌模式。".to_owned(),
                    },
                ));
            }
        }
        deliveries
    }
}

impl HostedGameLifecycle for TexasHoldemSession {
    const KIND: GameKind = GameKind::TexasHoldem;

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
        TexasHoldemRuleSet::MAX_PLAYERS
    }

    fn game_rules(&self) -> GameRules {
        let count = self
            .room
            .players
            .iter()
            .filter(|player| !player.left)
            .count();
        TexasHoldemRuleSet {
            squid_markers: self.rules.squid_markers.min(count.saturating_sub(1) as u8),
            ..self.rules
        }
        .into()
    }

    fn game_snapshot(&self, recipient: leocard_protocol::PlayerId) -> GameSnapshot {
        TexasHoldemSession::game_snapshot(self, recipient).into()
    }

    fn handle_game_command(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        command: GameCommand,
    ) -> Result<Vec<Delivery>, GameKind> {
        let GameCommand::TexasHoldem(command) = command else {
            return Err(command.kind());
        };
        Ok(match command {
            TexasHoldemCommand::SetAutoPlay { enabled } => {
                self.set_auto_play(connection, request_id, enabled)
            }
            TexasHoldemCommand::SetSpectatorWinRates { enabled } => {
                self.set_spectator_win_rates(connection, request_id, enabled)
            }
            TexasHoldemCommand::UpdateRules { rules } => {
                self.update_rules(connection, request_id, rules)
            }
            TexasHoldemCommand::Act { action } => self.act(connection, request_id, action),
        })
    }

    fn start_game(&mut self, connection: ConnectionId, request_id: RequestId) -> Vec<Delivery> {
        if self.room.player_id(connection).is_none() {
            return self.room.reject(
                connection,
                request_id,
                RejectReason::Player(PlayerViolation::NotJoined),
            );
        }
        if self.game.is_some() {
            return self.room.reject(
                connection,
                request_id,
                RejectReason::Game(GameViolation::GameAlreadyStarted),
            );
        }
        if self.room.host_connection != Some(connection) {
            return self.room.reject(
                connection,
                request_id,
                RejectReason::Room(RoomViolation::OnlyHostCanStart),
            );
        }
        let active_player_count = self
            .room
            .players
            .iter()
            .filter(|player| !player.left)
            .count();
        if active_player_count < usize::from(TexasHoldemRuleSet::MIN_PLAYERS) {
            return self.room.reject(
                connection,
                request_id,
                RejectReason::Room(RoomViolation::NotEnoughPlayers {
                    minimum: TexasHoldemRuleSet::MIN_PLAYERS,
                    actual: active_player_count as u8,
                }),
            );
        }
        if !self.rules.supports_player_count(active_player_count as u8) {
            return self.room.reject(
                connection,
                request_id,
                RejectReason::Game(GameViolation::InvalidRuleConfiguration),
            );
        }
        if self
            .room
            .players
            .iter()
            .any(|player| !player.left && player.seat.is_none())
        {
            return self.room.reject(
                connection,
                request_id,
                RejectReason::Room(RoomViolation::MustSelectSeat),
            );
        }
        let not_ready = self
            .room
            .players
            .iter()
            .filter(|player| !player.left)
            .filter(|player| !player.ready)
            .map(|player| player.id)
            .collect::<Vec<_>>();
        if !not_ready.is_empty() {
            return self.room.reject(
                connection,
                request_id,
                RejectReason::Room(RoomViolation::PlayersNotReady { players: not_ready }),
            );
        }
        // Keep lobby PlayerIds stable. Compaction is safe only once every recipient is
        // guaranteed to receive a private game snapshot carrying its reassigned `you`.
        self.room.remove_departed_players();
        match self.create_tournament() {
            Ok(()) => {
                self.room.bump_revision();
                self.broadcast_game(Some((connection, request_id)))
            }
            Err(_) => {
                self.room.remove_bots();
                self.room.reject(
                    connection,
                    request_id,
                    RejectReason::Game(GameViolation::InvalidRuleConfiguration),
                )
            }
        }
    }

    fn return_to_lobby(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
    ) -> Vec<Delivery> {
        if self.room.player_id(connection).is_none() {
            return self.room.reject(
                connection,
                request_id,
                RejectReason::Player(PlayerViolation::NotJoined),
            );
        }
        let Some(game) = self.game.as_ref() else {
            return self.room.reject(
                connection,
                request_id,
                RejectReason::Game(GameViolation::GameNotStarted),
            );
        };
        if !matches!(game.game().phase(), Phase::Complete(_)) || !self.tournament_complete() {
            return self.room.reject(
                connection,
                request_id,
                RejectReason::Game(GameViolation::GameNotFinished),
            );
        }
        self.game = None;
        self.match_profile_stats.clear();
        self.automatic_action_delay = None;
        self.room.remove_bots();
        let host = self.room.host_connection;
        for player in &mut self.room.players {
            player.ready = host == Some(player.connection);
            player.auto_play = false;
            if !player.connected {
                player.seat = None;
            }
        }
        let mut deck = build_deck(self.rules.short_deck);
        fastrand::shuffle(&mut deck);
        self.shuffled_deck = Some(deck);
        self.room.bump_revision();
        self.broadcast_lobby(Some((connection, request_id)))
    }

    fn play_again(&mut self, connection: ConnectionId, request_id: RequestId) -> Vec<Delivery> {
        let Some(player) = self.room.player_id(connection) else {
            return self.room.reject(
                connection,
                request_id,
                RejectReason::Player(PlayerViolation::NotJoined),
            );
        };
        let Some(game) = self.game.as_ref() else {
            return self.room.reject(
                connection,
                request_id,
                RejectReason::Game(GameViolation::GameNotStarted),
            );
        };
        if !matches!(game.game().phase(), Phase::Complete(_)) || self.tournament_complete() {
            return self.room.reject(
                connection,
                request_id,
                RejectReason::Game(GameViolation::GameNotFinished),
            );
        }
        if let Some(participant) = self.room.players.iter_mut().find(|p| p.id == player) {
            participant.ready = true;
        }
        self.room.bump_revision();
        let active = self
            .room
            .players
            .iter()
            .filter(|player| (player.connected || player.is_bot) && !player.left);
        let everyone_ready = active.clone().count() >= usize::from(TexasHoldemRuleSet::MIN_PLAYERS)
            && active.clone().all(|player| player.ready);
        if everyone_ready {
            for participant in &mut self.room.players {
                participant.ready = false;
            }
            let mut deck = build_deck(self.rules.short_deck);
            fastrand::shuffle(&mut deck);
            if self
                .game
                .as_mut()
                .is_none_or(|game| game.start_next_hand(deck).is_err())
            {
                return self.room.reject(
                    connection,
                    request_id,
                    RejectReason::Game(GameViolation::InvalidRuleConfiguration),
                );
            }
            let events = self.fold_disconnected_players();
            self.record_hand_started();
            self.record_profile_events(&events);
            self.finish_hand_if_needed(false);
            self.reset_automatic_action_delay_for_current_turn();
            self.room.bump_revision();
            let mut deliveries = self.broadcast_events(events);
            deliveries.extend(self.broadcast_game(Some((connection, request_id))));
            return deliveries;
        }
        self.broadcast_game(Some((connection, request_id)))
    }

    fn leave_room(&mut self, connection: ConnectionId, request_id: RequestId) -> Vec<Delivery> {
        let Some(index) = self
            .room
            .players
            .iter()
            .position(|player| player.connection == connection && !player.left)
        else {
            return self.room.reject(
                connection,
                request_id,
                RejectReason::Player(PlayerViolation::NotJoined),
            );
        };
        if self.room.host_connection == Some(connection) {
            return self.close_room(connection, request_id);
        }
        let id = self.room.players[index].id;
        let name = self.room.players[index].name.clone();
        self.room.players[index].ready = false;
        self.room.players[index].connected = false;
        self.room.players[index].left = true;
        if let Some(game) = self.game.as_mut() {
            let _ = game.set_connected(id, false);
        } else {
            self.room.players[index].seat = None;
        }
        let was_complete = self
            .game
            .as_ref()
            .is_some_and(|game| matches!(game.game().phase(), Phase::Complete(_)));
        let events = self.fold_disconnected_players();
        self.record_profile_events(&events);
        self.finish_hand_if_needed(was_complete);
        self.reset_automatic_action_delay_for_current_turn();
        self.room.bump_revision();
        let mut deliveries =
            vec![
                self.room
                    .delivery(connection, Some(request_id), ServerEvent::LeftRoom),
            ];
        deliveries.extend(
            self.room
                .broadcast_event(None, ServerEvent::PlayerLeft { name }),
        );
        deliveries.extend(self.broadcast_events(events));
        deliveries.extend(if self.game.is_some() {
            self.broadcast_game(None)
        } else {
            self.broadcast_lobby(None)
        });
        deliveries
    }

    fn after_join(&mut self, player: leocard_protocol::PlayerId) {
        if let Some(game) = self.game.as_mut() {
            let _ = game.set_connected(player, true);
        }
    }
}
