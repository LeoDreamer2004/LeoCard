use std::collections::HashSet;
use std::time::Duration;

use leocard_protocol::{
    ClientCommand, ClientMessage, GameCommand, GameEvent, GameKind, GameRules, GameSnapshot,
    GameViolation, LobbySnapshot, PlayerId, PlayerInteraction, PlayerInteractionKind,
    PlayerReferenceChange, RejectReason, RequestId, Revision, RoomId, ServerEvent,
    TABLE_SEAT_COUNT, TexasHoldemCommand, TexasHoldemEvent, TexasHoldemPhaseView,
    TexasHoldemProfileStats, TexasHoldemSnapshot,
};
use leocard_qigui523::reference_point_deltas;
use leocard_texas_holdem::{
    Action, Card, HandCategory, PassiveBot, Phase, RuleSet, build_deck, evaluate_player_hand,
};
use leocard_texas_holdem_adapter::{AdapterError, TablePlayer, TexasHoldemAdapter};

use crate::{
    AUTO_PLAY_DELAY, AutoPlayDelayState, ConnectionId, Delivery, HostError, RoomSession,
    new_match_id,
};

/// 复用 [`RoomSession`] 的德州扑克权威房间后端。
#[derive(Clone, Debug)]
pub struct TexasHoldemSession {
    room: RoomSession,
    rules: RuleSet,
    shuffled_deck: Option<Vec<Card>>,
    game: Option<TexasHoldemAdapter>,
    match_profile_stats: Vec<TexasHoldemProfileStats>,
    finished_reference_changes: Option<Vec<PlayerReferenceChange>>,
    auto_play_delay: Option<AutoPlayDelayState>,
}

impl TexasHoldemSession {
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
        let rules = RuleSet {
            player_count: TABLE_SEAT_COUNT,
            ..rules
        }
        .validate()?;
        validate_deck(rules.short_deck, &shuffled_deck)?;
        Ok(Self {
            room: RoomSession::new(room_id, host_port, usize::from(TABLE_SEAT_COUNT)),
            rules,
            shuffled_deck: Some(shuffled_deck),
            game: None,
            match_profile_stats: Vec::new(),
            finished_reference_changes: None,
            auto_play_delay: None,
        })
    }

    pub const fn room_id(&self) -> RoomId {
        self.room.room_id
    }

    pub const fn rules(&self) -> &RuleSet {
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

    pub fn heartbeat(&self) -> Vec<Delivery> {
        self.room
            .players
            .iter()
            .filter(|player| player.connected && !player.left)
            .map(|player| {
                self.room
                    .delivery(player.connection, None, ServerEvent::Heartbeat)
            })
            .collect()
    }

    /// 推进托管机器人的固定一秒行动延迟。
    pub fn advance_time(&mut self, elapsed: Duration) -> Vec<Delivery> {
        if elapsed.is_zero() || self.game.is_none() {
            return Vec::new();
        }
        let Some(player) = self.current_auto_play_player() else {
            self.auto_play_delay = None;
            return Vec::new();
        };
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
        let Some(events) = self.play_automatic_action() else {
            return Vec::new();
        };
        self.room.bump_revision();
        let mut deliveries = self.broadcast_events(events);
        deliveries.extend(self.broadcast_game(None));
        deliveries
    }

    pub fn disconnect(&mut self, connection: ConnectionId) -> Vec<Delivery> {
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
            return self
                .room
                .players
                .iter()
                .filter(|player| player.connected && !player.left)
                .map(|player| {
                    self.room
                        .delivery(player.connection, None, ServerEvent::RoomClosed)
                })
                .collect();
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
        self.reset_auto_play_delay_for_current_turn();
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
        if let Err(deliveries) = self.room.begin_request(connection, &message) {
            return deliveries;
        }
        let request_id = message.request_id;
        match message.command {
            ClientCommand::Join {
                name,
                reconnect_token,
                profile_id,
                reference_points,
                completed_games,
                game_profiles,
                identity_signature,
            } => {
                let joined = self.room.join(
                    connection,
                    request_id,
                    name,
                    reconnect_token,
                    profile_id,
                    reference_points,
                    completed_games,
                    game_profiles,
                    identity_signature,
                    self.game.is_some(),
                    TABLE_SEAT_COUNT,
                );
                match joined {
                    Ok(mut deliveries) => {
                        if let Some(game) = self.game.as_mut() {
                            let player = self
                                .room
                                .player_id(connection)
                                .expect("a successful join assigned a player");
                            let _ = game.set_connected(player, true);
                            deliveries.extend(self.broadcast_game(Some((connection, request_id))));
                        } else {
                            deliveries.extend(self.broadcast_lobby(Some((connection, request_id))));
                        }
                        deliveries
                    }
                    Err(reason) => self.room.reject(connection, request_id, reason),
                }
            }
            ClientCommand::SetAvatar { png } => {
                match self.room.set_avatar(connection, png, self.game.is_some()) {
                    Ok(mut deliveries) => {
                        deliveries.extend(self.broadcast_lobby(Some((connection, request_id))));
                        deliveries
                    }
                    Err(reason) => self.room.reject(connection, request_id, reason),
                }
            }
            ClientCommand::SelectSeat { seat } => {
                match self.room.select_seat(connection, seat, self.game.is_some()) {
                    Ok(()) => self.broadcast_lobby(Some((connection, request_id))),
                    Err(reason) => self.room.reject(connection, request_id, reason),
                }
            }
            ClientCommand::ConfigureBotSeat { seat, occupied } => {
                match self
                    .room
                    .configure_bot_seat(connection, seat, occupied, self.game.is_some())
                {
                    Ok(()) => self.broadcast_lobby(Some((connection, request_id))),
                    Err(reason) => self.room.reject(connection, request_id, reason),
                }
            }
            ClientCommand::SetReady { ready } => {
                match self.room.set_ready(connection, ready, self.game.is_some()) {
                    Ok(()) => self.broadcast_lobby(Some((connection, request_id))),
                    Err(reason) => self.room.reject(connection, request_id, reason),
                }
            }
            ClientCommand::Game(GameCommand::TexasHoldem(command)) => match command {
                TexasHoldemCommand::SetAutoPlay { enabled } => {
                    self.set_auto_play(connection, request_id, enabled)
                }
                TexasHoldemCommand::UpdateRules { rules } => {
                    self.update_rules(connection, request_id, rules)
                }
                TexasHoldemCommand::Act { action } => self.act(connection, request_id, action),
            },
            ClientCommand::Game(GameCommand::QiGui523(_)) => self.room.reject(
                connection,
                request_id,
                RejectReason::WrongGame {
                    expected: GameKind::TexasHoldem,
                    received: GameKind::QiGui523,
                },
            ),
            ClientCommand::Game(GameCommand::Shengji(_)) => self.room.reject(
                connection,
                request_id,
                RejectReason::WrongGame {
                    expected: GameKind::TexasHoldem,
                    received: GameKind::Shengji,
                },
            ),
            ClientCommand::Game(GameCommand::Uno(_)) => self.room.reject(
                connection,
                request_id,
                RejectReason::WrongGame {
                    expected: GameKind::TexasHoldem,
                    received: GameKind::Uno,
                },
            ),
            ClientCommand::StartGame => self.start_game(connection, request_id),
            ClientCommand::ReturnToLobby => self.return_to_lobby(connection, request_id),
            ClientCommand::PlayAgain => self.play_again(connection, request_id),
            ClientCommand::LeaveRoom => self.leave_room(connection, request_id),
            ClientCommand::CloseRoom => self.close_room(connection, request_id),
            ClientCommand::Interact { target, kind } => {
                self.interact(connection, request_id, target, kind)
            }
            ClientCommand::Chat { content } => self
                .room
                .chat(connection, request_id, content, self.game.is_some())
                .unwrap_or_else(|reason| self.room.reject(connection, request_id, reason)),
            ClientCommand::RequestSnapshot => self.snapshot(connection, request_id),
        }
    }

    fn update_rules(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        rules: RuleSet,
    ) -> Vec<Delivery> {
        if self.room.player_id(connection).is_none() {
            return self
                .room
                .reject(connection, request_id, RejectReason::NotJoined);
        }
        if self.game.is_some() {
            return self
                .room
                .reject(connection, request_id, RejectReason::GameAlreadyStarted);
        }
        if self.room.host_connection != Some(connection) {
            return self
                .room
                .reject(connection, request_id, RejectReason::OnlyHostCanConfigure);
        }
        let Ok(rules) = (RuleSet {
            player_count: TABLE_SEAT_COUNT,
            ..rules
        })
        .validate() else {
            return self.room.reject(
                connection,
                request_id,
                RejectReason::InvalidRuleConfiguration,
            );
        };
        if self.rules != rules {
            self.rules = rules;
            let host = self.room.host_connection;
            for player in &mut self.room.players {
                player.ready = host == Some(player.connection);
            }
            let mut deck = build_deck(rules.short_deck);
            fastrand::shuffle(&mut deck);
            self.shuffled_deck = Some(deck);
            self.room.bump_revision();
        }
        self.broadcast_lobby(Some((connection, request_id)))
    }

    fn set_auto_play(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        enabled: bool,
    ) -> Vec<Delivery> {
        let Some(player) = self.room.player_id(connection) else {
            return self
                .room
                .reject(connection, request_id, RejectReason::NotJoined);
        };
        let Some(game) = self.game.as_ref() else {
            return self
                .room
                .reject(connection, request_id, RejectReason::GameNotStarted);
        };
        if !matches!(game.game().phase(), Phase::Betting(_)) {
            return self.room.reject(
                connection,
                request_id,
                RejectReason::GameViolation(GameViolation::TexasHoldem(
                    leocard_protocol::TexasHoldemViolation::HandAlreadyComplete,
                )),
            );
        }

        let participant = self
            .room
            .players
            .iter_mut()
            .find(|participant| participant.id == player)
            .expect("joined player belongs to the room");
        let changed = participant.auto_play != enabled;
        participant.auto_play = enabled;
        if self
            .game
            .as_ref()
            .and_then(TexasHoldemAdapter::current_player)
            == Some(player)
        {
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
            self.room.bump_revision();
        }
        self.broadcast_game(Some((connection, request_id)))
    }

    fn start_game(&mut self, connection: ConnectionId, request_id: RequestId) -> Vec<Delivery> {
        if self.room.player_id(connection).is_none() {
            return self
                .room
                .reject(connection, request_id, RejectReason::NotJoined);
        }
        if self.game.is_some() {
            return self
                .room
                .reject(connection, request_id, RejectReason::GameAlreadyStarted);
        }
        if self.room.host_connection != Some(connection) {
            return self
                .room
                .reject(connection, request_id, RejectReason::OnlyHostCanStart);
        }
        let active_player_count = self
            .room
            .players
            .iter()
            .filter(|player| !player.left)
            .count();
        if active_player_count < usize::from(RuleSet::MIN_PLAYERS) {
            return self.room.reject(
                connection,
                request_id,
                RejectReason::NotEnoughPlayers {
                    minimum: RuleSet::MIN_PLAYERS,
                    actual: active_player_count as u8,
                },
            );
        }
        if self
            .room
            .players
            .iter()
            .any(|player| !player.left && player.seat.is_none())
        {
            return self
                .room
                .reject(connection, request_id, RejectReason::MustSelectSeat);
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
                RejectReason::PlayersNotReady { players: not_ready },
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
                #[cfg(feature = "developer")]
                self.room.remove_developer_bots();
                self.room.reject(
                    connection,
                    request_id,
                    RejectReason::InvalidRuleConfiguration,
                )
            }
        }
    }

    fn create_tournament(&mut self) -> Result<(), AdapterError> {
        let deck = self.shuffled_deck.take().unwrap_or_else(|| {
            let mut deck = build_deck(self.rules.short_deck);
            fastrand::shuffle(&mut deck);
            deck
        });
        let host = self
            .room
            .host_player_id()
            .expect("a started room has a host");
        let table_players = self
            .room
            .players
            .iter_mut()
            .filter(|player| !player.left)
            .map(|player| {
                player.ready = false;
                player.auto_play = player.is_bot;
                TablePlayer {
                    id: player.id,
                    profile_id: player.profile_id,
                    name: player.name.clone(),
                    avatar: player.avatar,
                    seat: player.seat.expect("start validation required every seat"),
                    connected: player.connected || player.is_bot,
                    reference_points: player.reference_points,
                    completed_games: player.completed_games,
                }
            })
            .collect::<Vec<_>>();
        let first_dealer = table_players[fastrand::usize(..table_players.len())].id;
        self.game = Some(TexasHoldemAdapter::new(
            new_match_id(),
            self.room.host_port,
            host,
            table_players,
            self.rules,
            first_dealer,
            deck,
        )?);
        self.match_profile_stats =
            vec![TexasHoldemProfileStats::default(); self.room.players.len()];
        self.record_hand_started();
        self.finished_reference_changes = None;
        self.auto_play_delay = None;
        self.reset_auto_play_delay_for_current_turn();
        Ok(())
    }

    fn act(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        action: Action,
    ) -> Vec<Delivery> {
        let Some(player) = self.room.player_id(connection) else {
            return self
                .room
                .reject(connection, request_id, RejectReason::NotJoined);
        };
        let was_complete = self
            .game
            .as_ref()
            .is_some_and(|game| matches!(game.game().phase(), Phase::Complete(_)));
        let Some(game) = self.game.as_mut() else {
            return self
                .room
                .reject(connection, request_id, RejectReason::GameNotStarted);
        };
        let mut events = match game.act(player, action) {
            Ok(events) => events,
            Err(AdapterError::Violation(violation)) => {
                return self.room.reject(
                    connection,
                    request_id,
                    RejectReason::GameViolation(GameViolation::TexasHoldem(violation)),
                );
            }
            Err(_) => {
                return self.room.reject(
                    connection,
                    request_id,
                    RejectReason::InvalidRuleConfiguration,
                );
            }
        };
        events.extend(self.fold_disconnected_players());
        self.record_profile_events(&events);
        self.finish_hand_if_needed(was_complete);
        self.reset_auto_play_delay_for_current_turn();
        self.room.bump_revision();
        let mut deliveries = self.broadcast_events(events);
        deliveries.extend(self.broadcast_game(Some((connection, request_id))));
        deliveries
    }

    fn return_to_lobby(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
    ) -> Vec<Delivery> {
        if self.room.player_id(connection).is_none() {
            return self
                .room
                .reject(connection, request_id, RejectReason::NotJoined);
        }
        let Some(game) = self.game.as_ref() else {
            return self
                .room
                .reject(connection, request_id, RejectReason::GameNotStarted);
        };
        if !matches!(game.game().phase(), Phase::Complete(_)) || !self.tournament_complete() {
            return self
                .room
                .reject(connection, request_id, RejectReason::GameNotFinished);
        }
        self.game = None;
        self.match_profile_stats.clear();
        self.auto_play_delay = None;
        #[cfg(feature = "developer")]
        self.room.remove_developer_bots();
        let host = self.room.host_connection;
        for player in &mut self.room.players {
            player.ready = host == Some(player.connection);
            player.auto_play = player.is_bot;
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
            return self
                .room
                .reject(connection, request_id, RejectReason::NotJoined);
        };
        let Some(game) = self.game.as_ref() else {
            return self
                .room
                .reject(connection, request_id, RejectReason::GameNotStarted);
        };
        if !matches!(game.game().phase(), Phase::Complete(_)) || self.tournament_complete() {
            return self
                .room
                .reject(connection, request_id, RejectReason::GameNotFinished);
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
        let everyone_ready = active.clone().count() >= usize::from(RuleSet::MIN_PLAYERS)
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
                    RejectReason::InvalidRuleConfiguration,
                );
            }
            let events = self.fold_disconnected_players();
            self.record_hand_started();
            self.record_profile_events(&events);
            self.finish_hand_if_needed(false);
            self.reset_auto_play_delay_for_current_turn();
            self.room.bump_revision();
            let mut deliveries = self.broadcast_events(events);
            deliveries.extend(self.broadcast_game(Some((connection, request_id))));
            return deliveries;
        }
        self.broadcast_game(Some((connection, request_id)))
    }

    fn tournament_complete(&self) -> bool {
        self.game.as_ref().is_some_and(|game| {
            matches!(game.game().phase(), Phase::Complete(_))
                && game
                    .game()
                    .players()
                    .iter()
                    .any(|player| player.stack() == 0)
        })
    }

    fn finish_hand_if_needed(&mut self, was_complete: bool) {
        if was_complete
            || !self
                .game
                .as_ref()
                .is_some_and(|game| matches!(game.game().phase(), Phase::Complete(_)))
        {
            return;
        }
        self.record_completed_hand();
        self.room.prepare_rematch();
        self.auto_play_delay = None;
        if self.tournament_complete() {
            self.apply_finished_reference_points();
        }
    }

    fn apply_finished_reference_points(&mut self) {
        if self.finished_reference_changes.is_some() {
            return;
        }
        let Some(game) = self.game.as_ref() else {
            return;
        };
        let standings = game
            .players()
            .iter()
            .zip(game.game().players())
            .map(|(participant, state)| (participant.id, state.stack()))
            .collect::<Vec<_>>();
        let scores = standings
            .iter()
            .map(|(_, stack)| *stack)
            .collect::<Vec<_>>();
        let deltas = reference_point_deltas(&scores)
            .expect("a Texas Hold'em table always contains between three and six players");
        let mut changes = Vec::with_capacity(standings.len());
        let match_profile_stats = self.match_profile_stats.clone();
        for ((player_id, final_chips), delta) in standings.into_iter().zip(deltas) {
            let current_profile_stats = match_profile_stats.get(usize::from(player_id.0));
            let participant = self
                .room
                .players
                .iter_mut()
                .find(|participant| participant.id == player_id)
                .expect("an adapter participant belongs to the room");
            participant.reference_points = participant
                .reference_points
                .saturating_add(i32::from(delta));
            participant.completed_games = participant.completed_games.saturating_add(1);
            let placement = 1 + scores.iter().filter(|other| **other > final_chips).count();
            let aggregate = participant
                .game_profiles
                .texas_holdem
                .get_or_insert_with(TexasHoldemProfileStats::default);
            aggregate.completed_games = aggregate.completed_games.saturating_add(1);
            aggregate.total_reference_delta = aggregate
                .total_reference_delta
                .saturating_add(i64::from(delta));
            aggregate.total_final_chips = aggregate
                .total_final_chips
                .saturating_add(u64::from(final_chips));
            if let Some(count) = aggregate.placement_counts.get_mut(placement - 1) {
                *count = count.saturating_add(1);
            }
            if let Some(current) = current_profile_stats {
                merge_texas_holdem_profile_stats(aggregate, current);
            }
            changes.push(PlayerReferenceChange {
                player: player_id,
                profile_id: participant.profile_id,
                delta,
            });
        }
        self.finished_reference_changes = Some(changes);
    }

    fn leave_room(&mut self, connection: ConnectionId, request_id: RequestId) -> Vec<Delivery> {
        let Some(index) = self
            .room
            .players
            .iter()
            .position(|player| player.connection == connection && !player.left)
        else {
            return self
                .room
                .reject(connection, request_id, RejectReason::NotJoined);
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
        self.reset_auto_play_delay_for_current_turn();
        self.room.bump_revision();
        let mut deliveries =
            vec![
                self.room
                    .delivery(connection, Some(request_id), ServerEvent::LeftRoom),
            ];
        deliveries.extend(
            self.room
                .players
                .iter()
                .filter(|player| player.connected && !player.left)
                .map(|player| {
                    self.room.delivery(
                        player.connection,
                        None,
                        ServerEvent::PlayerLeft { name: name.clone() },
                    )
                }),
        );
        deliveries.extend(self.broadcast_events(events));
        deliveries.extend(if self.game.is_some() {
            self.broadcast_game(None)
        } else {
            self.broadcast_lobby(None)
        });
        deliveries
    }

    fn close_room(&mut self, connection: ConnectionId, request_id: RequestId) -> Vec<Delivery> {
        self.room
            .close_room(connection, request_id)
            .unwrap_or_else(|reason| self.room.reject(connection, request_id, reason))
    }

    fn interact(
        &self,
        connection: ConnectionId,
        request_id: RequestId,
        target: PlayerId,
        kind: PlayerInteractionKind,
    ) -> Vec<Delivery> {
        let Some(source) = self.room.player_id(connection) else {
            return self
                .room
                .reject(connection, request_id, RejectReason::NotJoined);
        };
        let Some(game) = self.game.as_ref() else {
            return self
                .room
                .reject(connection, request_id, RejectReason::GameNotStarted);
        };
        if !matches!(game.game().phase(), Phase::Betting(_)) {
            return self.room.reject(
                connection,
                request_id,
                RejectReason::GameViolation(GameViolation::TexasHoldem(
                    leocard_protocol::TexasHoldemViolation::HandAlreadyComplete,
                )),
            );
        }
        if source == target
            || !self
                .room
                .players
                .iter()
                .any(|player| player.id == target && !player.left)
        {
            return self.room.reject(
                connection,
                request_id,
                RejectReason::GameViolation(GameViolation::TexasHoldem(
                    leocard_protocol::TexasHoldemViolation::InvalidPlayer,
                )),
            );
        }
        let interaction = PlayerInteraction {
            source,
            target,
            kind,
            seed: fastrand::u32(..),
        };
        self.room
            .players
            .iter()
            .filter(|player| player.connected && !player.left)
            .map(|player| {
                self.room.delivery(
                    player.connection,
                    (player.connection == connection).then_some(request_id),
                    ServerEvent::PlayerInteraction(interaction),
                )
            })
            .collect()
    }

    fn snapshot(&self, connection: ConnectionId, request_id: RequestId) -> Vec<Delivery> {
        let Some(player) = self.room.player_id(connection) else {
            return self
                .room
                .reject(connection, request_id, RejectReason::NotJoined);
        };
        let event = if self.game.is_some() {
            ServerEvent::GameSnapshot(GameSnapshot::TexasHoldem(self.game_snapshot(player)))
        } else {
            ServerEvent::LobbySnapshot(self.lobby_snapshot())
        };
        vec![self.room.delivery(connection, Some(request_id), event)]
    }

    fn current_auto_play_player(&self) -> Option<PlayerId> {
        let current = self
            .game
            .as_ref()
            .and_then(TexasHoldemAdapter::current_player)?;
        self.room
            .players
            .iter()
            .find(|player| player.id == current)
            .is_some_and(|player| {
                !player.left && (player.connected || player.is_bot) && player.auto_play
            })
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

    fn play_automatic_action(&mut self) -> Option<Vec<TexasHoldemEvent>> {
        let (player, action) = {
            let game = self.game.as_ref()?;
            let player = game.current_player()?;
            let core_player = game.game().current_player()?;
            let action = PassiveBot::choose_for_game(game.game(), core_player)?;
            (player, action)
        };
        let was_complete = self
            .game
            .as_ref()
            .is_some_and(|game| matches!(game.game().phase(), Phase::Complete(_)));
        let mut events = self.game.as_mut()?.act(player, action).ok()?;
        events.extend(self.fold_disconnected_players());
        self.record_profile_events(&events);
        self.finish_hand_if_needed(was_complete);
        self.reset_auto_play_delay_for_current_turn();
        Some(events)
    }

    fn record_profile_events(&mut self, events: &[TexasHoldemEvent]) {
        for event in events {
            let TexasHoldemEvent::ActionApplied {
                player,
                action,
                amount,
            } = event
            else {
                continue;
            };
            let Some(stats) = self.match_profile_stats.get_mut(usize::from(player.0)) else {
                continue;
            };
            match action {
                Action::PostBlind => continue,
                Action::Fold => {
                    stats.voluntary_actions = stats.voluntary_actions.saturating_add(1);
                    stats.hands_folded = stats.hands_folded.saturating_add(1);
                }
                Action::Check => {
                    stats.voluntary_actions = stats.voluntary_actions.saturating_add(1);
                    stats.check_actions = stats.check_actions.saturating_add(1);
                }
                Action::Call => {
                    stats.voluntary_actions = stats.voluntary_actions.saturating_add(1);
                    record_wager(stats, *amount);
                }
                Action::RaiseTo(_) => {
                    stats.voluntary_actions = stats.voluntary_actions.saturating_add(1);
                    stats.raise_actions = stats.raise_actions.saturating_add(1);
                    record_wager(stats, *amount);
                }
                Action::AllIn => {
                    stats.voluntary_actions = stats.voluntary_actions.saturating_add(1);
                    stats.all_in_actions = stats.all_in_actions.saturating_add(1);
                    record_wager(stats, *amount);
                }
            }
        }
    }

    fn record_hand_started(&mut self) {
        let Some(game) = self.game.as_ref() else {
            return;
        };
        for (participant, state) in game.players().iter().zip(game.game().players()) {
            if !state.hole_cards().is_empty()
                && let Some(stats) = self
                    .match_profile_stats
                    .get_mut(usize::from(participant.id.0))
            {
                stats.hands_played = stats.hands_played.saturating_add(1);
            }
        }
    }

    fn record_completed_hand(&mut self) {
        let Some(game) = self.game.as_ref() else {
            return;
        };
        let Phase::Complete(result) = game.game().phase() else {
            return;
        };
        if !result.showdown {
            return;
        }
        let categories = game
            .players()
            .iter()
            .zip(game.game().players())
            .filter(|(_, state)| !state.folded())
            .filter_map(|(participant, state)| {
                evaluate_player_hand(state.hole_cards(), game.game().community(), game.rules())
                    .ok()
                    .map(|hand| (participant.id, hand.category()))
            })
            .collect::<Vec<_>>();
        for (player, category) in categories {
            if let Some(stats) = self.match_profile_stats.get_mut(usize::from(player.0)) {
                let count = &mut stats.hand_category_counts[hand_category_index(category)];
                *count = count.saturating_add(1);
            }
        }
    }

    fn fold_disconnected_players(&mut self) -> Vec<TexasHoldemEvent> {
        let mut events = Vec::new();
        while let Some(current) = self
            .game
            .as_ref()
            .and_then(TexasHoldemAdapter::current_player)
        {
            let disconnected = self
                .room
                .players
                .iter()
                .find(|player| player.id == current)
                .is_some_and(|player| (!player.connected && !player.is_bot) || player.left);
            if !disconnected {
                break;
            }
            let Some(game) = self.game.as_mut() else {
                break;
            };
            let automatic_action = if game.game().blind_to_post().is_some() {
                Action::PostBlind
            } else {
                Action::Fold
            };
            match game.act(current, automatic_action) {
                Ok(auto_events) => events.extend(auto_events),
                Err(_) => break,
            }
        }
        events
    }

    fn lobby_snapshot(&self) -> LobbySnapshot {
        self.room
            .lobby_snapshot(GameKind::TexasHoldem, GameRules::TexasHoldem(self.rules))
    }

    fn broadcast_lobby(&self, origin: Option<(ConnectionId, RequestId)>) -> Vec<Delivery> {
        self.room.broadcast_lobby(
            GameKind::TexasHoldem,
            GameRules::TexasHoldem(self.rules),
            origin,
        )
    }

    fn broadcast_game(&self, origin: Option<(ConnectionId, RequestId)>) -> Vec<Delivery> {
        assert!(self.game.is_some(), "game broadcast requires a game");
        self.room
            .players
            .iter()
            .filter(|player| player.connected && !player.left)
            .map(|player| {
                let reply = origin
                    .filter(|(connection, _)| *connection == player.connection)
                    .map(|(_, request)| request);
                let snapshot = self.game_snapshot(player.id);
                self.room.delivery(
                    player.connection,
                    reply,
                    ServerEvent::GameSnapshot(GameSnapshot::TexasHoldem(snapshot)),
                )
            })
            .collect()
    }

    fn game_snapshot(&self, recipient: PlayerId) -> TexasHoldemSnapshot {
        let mut snapshot = self
            .game
            .as_ref()
            .expect("a game snapshot requires an active game")
            .snapshot(recipient)
            .expect("an active room participant belongs to the adapter");
        for state in &mut snapshot.players {
            if let Some(participant) = self
                .room
                .players
                .iter()
                .find(|player| player.id == state.id)
            {
                state.connected =
                    (participant.connected || participant.is_bot) && !participant.left;
                state.auto_play = participant.auto_play;
                state.ready = participant.ready;
                state.reference_points = participant.reference_points;
                state.completed_games = participant.completed_games;
                state.game_profiles.clone_from(&participant.game_profiles);
            }
        }
        if let TexasHoldemPhaseView::HandComplete {
            tournament_complete,
            reference_changes,
            ..
        } = &mut snapshot.phase
        {
            *tournament_complete = self.tournament_complete();
            *reference_changes = self.finished_reference_changes.clone().unwrap_or_default();
        }
        snapshot
    }

    fn broadcast_events(&self, events: Vec<TexasHoldemEvent>) -> Vec<Delivery> {
        events
            .into_iter()
            .flat_map(|event| {
                self.room
                    .players
                    .iter()
                    .filter(|player| player.connected && !player.left)
                    .map(move |player| {
                        self.room.delivery(
                            player.connection,
                            None,
                            ServerEvent::GameEvent(GameEvent::TexasHoldem(event.clone())),
                        )
                    })
            })
            .collect()
    }
}

fn record_wager(stats: &mut TexasHoldemProfileStats, amount: u32) {
    if amount == 0 {
        return;
    }
    stats.wagered_chips = stats.wagered_chips.saturating_add(u64::from(amount));
    stats.wager_actions = stats.wager_actions.saturating_add(1);
}

const fn hand_category_index(category: HandCategory) -> usize {
    match category {
        HandCategory::HighCard => 0,
        HandCategory::OnePair => 1,
        HandCategory::TwoPair => 2,
        HandCategory::ThreeOfAKind => 3,
        HandCategory::Straight => 4,
        HandCategory::Flush => 5,
        HandCategory::FullHouse => 6,
        HandCategory::FourOfAKind => 7,
        HandCategory::StraightFlush => 8,
        HandCategory::RoyalFlush => 9,
    }
}

fn merge_texas_holdem_profile_stats(
    aggregate: &mut TexasHoldemProfileStats,
    current: &TexasHoldemProfileStats,
) {
    aggregate.wagered_chips = aggregate
        .wagered_chips
        .saturating_add(current.wagered_chips);
    aggregate.wager_actions = aggregate
        .wager_actions
        .saturating_add(current.wager_actions);
    aggregate.voluntary_actions = aggregate
        .voluntary_actions
        .saturating_add(current.voluntary_actions);
    aggregate.check_actions = aggregate
        .check_actions
        .saturating_add(current.check_actions);
    aggregate.raise_actions = aggregate
        .raise_actions
        .saturating_add(current.raise_actions);
    aggregate.all_in_actions = aggregate
        .all_in_actions
        .saturating_add(current.all_in_actions);
    aggregate.hands_played = aggregate.hands_played.saturating_add(current.hands_played);
    aggregate.hands_folded = aggregate.hands_folded.saturating_add(current.hands_folded);
    for (aggregate, current) in aggregate
        .hand_category_counts
        .iter_mut()
        .zip(current.hand_category_counts)
    {
        *aggregate = aggregate.saturating_add(current);
    }
}

fn validate_deck(short_deck: bool, deck: &[Card]) -> Result<(), HostError> {
    let expected_deck = build_deck(short_deck);
    if deck.len() != expected_deck.len() {
        return Err(HostError::InvalidDeckSize {
            expected: expected_deck.len(),
            actual: deck.len(),
        });
    }
    let expected = expected_deck.into_iter().collect::<HashSet<_>>();
    let actual = deck.iter().copied().collect::<HashSet<_>>();
    if actual.len() != deck.len() || actual != expected {
        return Err(HostError::InvalidDeckContents);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use ed25519_dalek::{Signer, SigningKey};
    use leocard_protocol::{
        ClientCommand, GameCommand, GameSnapshot, ProfileId, ReconnectToken, RequestId,
        ServerEvent, TexasHoldemCommand, join_identity_payload,
    };
    use leocard_texas_holdem::{Action, Phase, Rank, RuleSet, Suit, build_deck};

    use super::*;

    const ROOM: RoomId = RoomId(9527);
    const HOST_CONNECTION: ConnectionId = ConnectionId(10);
    const SECOND_CONNECTION: ConnectionId = ConnectionId(20);
    const THIRD_CONNECTION: ConnectionId = ConnectionId(30);

    fn message(request: u64, command: ClientCommand) -> ClientMessage {
        ClientMessage::new(ROOM, RequestId(request), command)
    }

    fn join_command(name: &str, token: u64) -> ClientCommand {
        let mut secret = [0; 32];
        secret[..8].copy_from_slice(&token.to_be_bytes());
        secret[8] = 9;
        let key = SigningKey::from_bytes(&secret);
        let reconnect_token = ReconnectToken(token);
        let profile_id = ProfileId(key.verifying_key().to_bytes());
        let game_profiles = leocard_protocol::PlayerGameProfiles::default();
        let payload = join_identity_payload(ROOM, reconnect_token, name, 0, 0, &game_profiles);
        ClientCommand::Join {
            name: name.to_owned(),
            reconnect_token,
            profile_id,
            reference_points: 0,
            completed_games: 0,
            game_profiles,
            identity_signature: key.sign(&payload).to_bytes().to_vec(),
        }
    }

    fn session_waiting_for_blinds() -> TexasHoldemSession {
        let mut session = TexasHoldemSession::new_with_host_port(
            ROOM,
            52301,
            RuleSet::default(),
            build_deck(false),
        )
        .unwrap();
        for (connection, name, token) in [
            (HOST_CONNECTION, "房主", 1),
            (SECOND_CONNECTION, "玩家二", 2),
            (THIRD_CONNECTION, "玩家三", 3),
        ] {
            session.handle(connection, message(1, join_command(name, token)));
        }
        for connection in [SECOND_CONNECTION, THIRD_CONNECTION] {
            session.handle(
                connection,
                message(2, ClientCommand::SetReady { ready: true }),
            );
        }
        let deliveries = session.handle(HOST_CONNECTION, message(2, ClientCommand::StartGame));
        assert!(deliveries.iter().any(|delivery| matches!(
            delivery.message.event,
            ServerEvent::GameSnapshot(GameSnapshot::TexasHoldem(_))
        )));
        session
    }

    fn started_session() -> TexasHoldemSession {
        let mut session = session_waiting_for_blinds();
        for _ in 0..2 {
            let player = session.game().unwrap().current_player().unwrap();
            session
                .game
                .as_mut()
                .unwrap()
                .act(player, Action::PostBlind)
                .unwrap();
        }
        session
    }

    #[test]
    fn lobby_start_waits_for_both_blind_actions() {
        let mut session = session_waiting_for_blinds();
        let first = session
            .game()
            .unwrap()
            .snapshot(session.room.players[0].id)
            .unwrap()
            .blind_to_post
            .unwrap();
        let connection = connection_for(&session, first.player);
        let delivered = session.handle(
            connection,
            message(
                3,
                ClientCommand::Game(GameCommand::TexasHoldem(TexasHoldemCommand::Act {
                    action: Action::PostBlind,
                })),
            ),
        );
        assert!(delivered.iter().any(|delivery| matches!(
            delivery.message.event,
            ServerEvent::GameEvent(GameEvent::TexasHoldem(TexasHoldemEvent::ActionApplied {
                action: Action::PostBlind,
                ..
            }))
        )));
        assert!(session.game().unwrap().game().blind_to_post().is_some());
    }

    #[test]
    fn texas_auto_play_waits_one_second_then_uses_the_passive_bot_action() {
        let mut session = session_waiting_for_blinds();
        let current = session.game().unwrap().current_player().unwrap();
        let connection = connection_for(&session, current);
        session.handle(
            connection,
            message(
                3,
                ClientCommand::Game(GameCommand::TexasHoldem(TexasHoldemCommand::SetAutoPlay {
                    enabled: true,
                })),
            ),
        );

        let snapshot = session.game_snapshot(current);
        assert!(
            snapshot
                .players
                .iter()
                .find(|player| player.id == current)
                .unwrap()
                .auto_play
        );
        assert!(session.advance_time(Duration::from_millis(999)).is_empty());
        let deliveries = session.advance_time(Duration::from_millis(1));
        assert!(deliveries.iter().any(|delivery| matches!(
            delivery.message.event,
            ServerEvent::GameEvent(GameEvent::TexasHoldem(
                TexasHoldemEvent::ActionApplied {
                    player,
                    action: Action::PostBlind,
                    ..
                }
            )) if player == current
        )));
    }

    #[cfg(feature = "developer")]
    #[test]
    fn developer_host_right_click_commands_add_default_texas_bots() {
        let mut session = TexasHoldemSession::new_with_host_port(
            ROOM,
            52301,
            RuleSet::default(),
            build_deck(false),
        )
        .unwrap();
        session.handle(HOST_CONNECTION, message(1, join_command("房主", 1)));
        let rejected = session.handle(HOST_CONNECTION, message(2, ClientCommand::StartGame));
        assert!(rejected.iter().any(|delivery| matches!(
            delivery.message.event,
            ServerEvent::Rejected {
                reason: RejectReason::NotEnoughPlayers { .. }
            }
        )));
        let empty_seats = (0..TABLE_SEAT_COUNT)
            .map(leocard_protocol::SeatId)
            .filter(|seat| {
                session
                    .room
                    .players
                    .iter()
                    .all(|player| player.seat != Some(*seat))
            })
            .take(2)
            .collect::<Vec<_>>();
        for (index, seat) in empty_seats.into_iter().enumerate() {
            session.handle(
                HOST_CONNECTION,
                message(
                    3 + index as u64,
                    ClientCommand::ConfigureBotSeat {
                        seat,
                        occupied: true,
                    },
                ),
            );
        }
        let deliveries = session.handle(HOST_CONNECTION, message(5, ClientCommand::StartGame));

        assert!(deliveries.iter().any(|delivery| matches!(
            delivery.message.event,
            ServerEvent::GameSnapshot(GameSnapshot::TexasHoldem(_))
        )));
        assert_eq!(session.room.players.len(), RuleSet::MIN_PLAYERS as usize);
        assert_eq!(
            session
                .room
                .players
                .iter()
                .filter(|player| player.is_bot)
                .count(),
            RuleSet::MIN_PLAYERS as usize - 1
        );
        for bot in session.room.players.iter().filter(|player| player.is_bot) {
            assert_eq!(bot.profile_id, ProfileId([0; 32]));
            assert_eq!(bot.avatar, None);
            assert_eq!(bot.reference_points, 0);
            assert_eq!(bot.completed_games, 0);
            assert!(bot.auto_play);
        }
        let snapshot = session.game_snapshot(session.room.host_player_id().unwrap());
        assert!(
            snapshot
                .players
                .iter()
                .filter(|player| player.auto_play)
                .count()
                >= 2
        );
    }

    fn connection_for(session: &TexasHoldemSession, player: PlayerId) -> ConnectionId {
        session
            .room
            .players
            .iter()
            .find(|participant| participant.id == player)
            .unwrap()
            .connection
    }

    #[test]
    fn common_lobby_starts_texas_and_each_snapshot_has_only_its_own_hole_cards() {
        let session = started_session();
        assert_eq!(session.rules().player_count, TABLE_SEAT_COUNT);
        let game = session.game().unwrap();
        let snapshots = session
            .room
            .players
            .iter()
            .map(|player| game.snapshot(player.id).unwrap())
            .collect::<Vec<_>>();
        assert!(snapshots.iter().all(|snapshot| {
            snapshot.your_hole_cards.len() == 2 && snapshot.revealed_hands.is_empty()
        }));
        assert_ne!(snapshots[0].your_hole_cards, snapshots[1].your_hole_cards);
        assert!(snapshots.iter().all(|snapshot| snapshot.host_port == 52301));
        assert_eq!(snapshots[0].players.len(), 3);
        assert_eq!(snapshots[0].pot, 3);
    }

    #[test]
    fn protocol_actions_use_the_adapter_and_rejections_leave_state_unchanged() {
        let mut session = started_session();
        let current = session.game().unwrap().current_player().unwrap();
        let connection = connection_for(&session, current);
        let before = session.game().unwrap().game().clone();
        let rejected = session.handle(
            connection,
            message(
                3,
                ClientCommand::Game(GameCommand::TexasHoldem(TexasHoldemCommand::Act {
                    action: Action::Check,
                })),
            ),
        );
        assert!(rejected.iter().any(|delivery| matches!(
            delivery.message.event,
            ServerEvent::Rejected {
                reason: RejectReason::GameViolation(GameViolation::TexasHoldem(
                    leocard_protocol::TexasHoldemViolation::CannotCheckWhileFacingBet { .. }
                ))
            }
        )));
        assert_eq!(session.game().unwrap().game(), &before);

        let accepted = session.handle(
            connection,
            message(
                4,
                ClientCommand::Game(GameCommand::TexasHoldem(TexasHoldemCommand::Act {
                    action: Action::Call,
                })),
            ),
        );
        assert!(accepted.iter().any(|delivery| matches!(
            delivery.message.event,
            ServerEvent::GameEvent(GameEvent::TexasHoldem(
                TexasHoldemEvent::ActionApplied {
                    player,
                    action: Action::Call,
                    ..
                }
            )) if player == current
        )));
    }

    #[test]
    fn all_players_ready_start_the_next_hand_with_stacks_and_rotated_dealer_preserved() {
        let mut session = started_session();
        let first_dealer = session.game().unwrap().game().dealer();
        let mut requests = HashMap::from([
            (HOST_CONNECTION, 2_u64),
            (SECOND_CONNECTION, 2_u64),
            (THIRD_CONNECTION, 2_u64),
        ]);
        while matches!(session.game().unwrap().game().phase(), Phase::Betting(_)) {
            let player = session.game().unwrap().current_player().unwrap();
            let connection = connection_for(&session, player);
            let request = requests.get_mut(&connection).unwrap();
            *request += 1;
            session.handle(
                connection,
                message(
                    *request,
                    ClientCommand::Game(GameCommand::TexasHoldem(TexasHoldemCommand::Act {
                        action: Action::Fold,
                    })),
                ),
            );
        }
        assert!(matches!(
            session.game().unwrap().game().phase(),
            Phase::Complete(_)
        ));
        for (index, connection) in [HOST_CONNECTION, SECOND_CONNECTION, THIRD_CONNECTION]
            .into_iter()
            .enumerate()
        {
            let request = requests.get_mut(&connection).unwrap();
            *request += 1;
            session.handle(connection, message(*request, ClientCommand::PlayAgain));
            if index < 2 {
                let snapshot = session.game_snapshot(session.room.players[0].id);
                assert!(matches!(
                    snapshot.phase,
                    TexasHoldemPhaseView::HandComplete { .. }
                ));
                assert_eq!(
                    snapshot
                        .players
                        .iter()
                        .filter(|player| player.ready)
                        .count(),
                    index + 1
                );
            }
        }
        let game = session.game().unwrap().game();
        assert_eq!(game.hand_number(), 1);
        assert_ne!(game.dealer(), first_dealer);
        assert!(matches!(game.phase(), Phase::Betting(_)));
    }

    #[test]
    fn first_busted_player_finishes_the_tournament_and_applies_shared_rating_once() {
        let prefix = [
            Card::new(Suit::Spade, Rank::King),
            Card::new(Suit::Spade, Rank::Queen),
            Card::new(Suit::Spade, Rank::Ace),
            Card::new(Suit::Heart, Rank::King),
            Card::new(Suit::Heart, Rank::Queen),
            Card::new(Suit::Heart, Rank::Ace),
            Card::new(Suit::Club, Rank::Two),
            Card::new(Suit::Diamond, Rank::Three),
            Card::new(Suit::Spade, Rank::Seven),
            Card::new(Suit::Club, Rank::Eight),
            Card::new(Suit::Diamond, Rank::Nine),
        ];
        let deck = prefix
            .into_iter()
            .chain(
                build_deck(false)
                    .into_iter()
                    .filter(|card| !prefix.contains(card)),
            )
            .collect();
        let mut session = TexasHoldemSession::new_with_host_port(
            ROOM,
            52301,
            RuleSet {
                starting_chips: 5,
                ..RuleSet::default()
            },
            deck,
        )
        .unwrap();
        for (connection, name, token) in [
            (HOST_CONNECTION, "房主", 1),
            (SECOND_CONNECTION, "玩家二", 2),
            (THIRD_CONNECTION, "玩家三", 3),
        ] {
            session.handle(connection, message(1, join_command(name, token)));
        }
        for connection in [SECOND_CONNECTION, THIRD_CONNECTION] {
            session.handle(
                connection,
                message(2, ClientCommand::SetReady { ready: true }),
            );
        }
        session.handle(HOST_CONNECTION, message(2, ClientCommand::StartGame));
        let mut request = 3;
        while session.game().unwrap().game().blind_to_post().is_some() {
            let player = session.game().unwrap().current_player().unwrap();
            let connection = connection_for(&session, player);
            session.handle(
                connection,
                message(
                    request,
                    ClientCommand::Game(GameCommand::TexasHoldem(TexasHoldemCommand::Act {
                        action: Action::PostBlind,
                    })),
                ),
            );
            request += 1;
        }
        while matches!(session.game().unwrap().game().phase(), Phase::Betting(_)) {
            let player = session.game().unwrap().current_player().unwrap();
            let connection = connection_for(&session, player);
            session.handle(
                connection,
                message(
                    request,
                    ClientCommand::Game(GameCommand::TexasHoldem(TexasHoldemCommand::Act {
                        action: Action::AllIn,
                    })),
                ),
            );
            request += 1;
        }
        let snapshot = session.game_snapshot(session.room.players[0].id);
        let TexasHoldemPhaseView::HandComplete {
            tournament_complete,
            reference_changes,
            ..
        } = snapshot.phase
        else {
            panic!("all-in showdown should complete the hand");
        };
        assert!(
            tournament_complete,
            "final stacks: {:?}",
            snapshot
                .players
                .iter()
                .map(|player| player.stack)
                .collect::<Vec<_>>()
        );
        assert_eq!(reference_changes.len(), 3);
        assert!(snapshot.players.iter().any(|player| player.stack == 0));
        for player in &snapshot.players {
            let stats = player.game_profiles.texas_holdem.as_ref().unwrap();
            let delta = reference_changes
                .iter()
                .find(|change| change.player == player.id)
                .unwrap()
                .delta;
            let placement = 1 + snapshot
                .players
                .iter()
                .filter(|other| other.stack > player.stack)
                .count();
            assert_eq!(stats.completed_games, 1);
            assert_eq!(stats.total_reference_delta, i64::from(delta));
            assert_eq!(stats.total_final_chips, u64::from(player.stack));
            assert_eq!(stats.placement_counts[placement - 1], 1);
            assert_eq!(stats.hands_played, 1);
            assert_eq!(stats.all_in_actions, 1);
            assert_eq!(stats.wager_actions, 1);
            assert_eq!(stats.hand_category_counts.iter().sum::<u32>(), 1);
        }
        let points_after = session
            .room
            .players
            .iter()
            .map(|player| player.reference_points)
            .collect::<Vec<_>>();
        session.apply_finished_reference_points();
        assert_eq!(
            session
                .room
                .players
                .iter()
                .map(|player| player.reference_points)
                .collect::<Vec<_>>(),
            points_after
        );
    }

    #[test]
    fn texas_profile_action_statistics_ignore_blinds_and_zero_value_actions() {
        let mut session =
            TexasHoldemSession::new(ROOM, RuleSet::default(), build_deck(false)).unwrap();
        session.match_profile_stats = vec![TexasHoldemProfileStats::default(); 2];
        session.record_profile_events(&[
            TexasHoldemEvent::ActionApplied {
                player: PlayerId(0),
                action: Action::PostBlind,
                amount: 2,
            },
            TexasHoldemEvent::ActionApplied {
                player: PlayerId(0),
                action: Action::Check,
                amount: 0,
            },
            TexasHoldemEvent::ActionApplied {
                player: PlayerId(0),
                action: Action::RaiseTo(12),
                amount: 10,
            },
            TexasHoldemEvent::ActionApplied {
                player: PlayerId(0),
                action: Action::Fold,
                amount: 0,
            },
        ]);

        let stats = &session.match_profile_stats[0];
        assert_eq!(stats.voluntary_actions, 3);
        assert_eq!(stats.check_actions, 1);
        assert_eq!(stats.raise_actions, 1);
        assert_eq!(stats.hands_folded, 1);
        assert_eq!(stats.wager_actions, 1);
        assert_eq!(stats.wagered_chips, 10);
    }

    #[test]
    fn disconnecting_the_current_guest_immediately_folds_them() {
        let mut session = started_session();
        let current = session.game().unwrap().current_player().unwrap();
        let connection = connection_for(&session, current);
        if connection == HOST_CONNECTION {
            // 先合法行动一次，确保测试目标是可断线而不关闭房间的客人。
            session.handle(
                connection,
                message(
                    3,
                    ClientCommand::Game(GameCommand::TexasHoldem(TexasHoldemCommand::Act {
                        action: Action::Call,
                    })),
                ),
            );
        }
        let guest = session.game().unwrap().current_player().unwrap();
        let guest_connection = connection_for(&session, guest);
        assert_ne!(guest_connection, HOST_CONNECTION);
        let deliveries = session.disconnect(guest_connection);
        assert!(!deliveries.is_empty());
        let core_index = session
            .game()
            .unwrap()
            .players()
            .iter()
            .position(|player| player.id == guest)
            .unwrap();
        assert!(session.game().unwrap().game().players()[core_index].folded());
    }

    #[test]
    fn texas_rules_are_visible_in_the_shared_lobby_snapshot() {
        let mut session = TexasHoldemSession::new(
            ROOM,
            RuleSet {
                short_deck: true,
                ignore_kickers: true,
                omaha: true,
                ..RuleSet::default()
            },
            build_deck(true),
        )
        .unwrap();
        let deliveries = session.handle(HOST_CONNECTION, message(1, join_command("房主", 11)));
        let lobby = deliveries
            .iter()
            .find_map(|delivery| match &delivery.message.event {
                ServerEvent::LobbySnapshot(snapshot) => Some(snapshot),
                _ => None,
            })
            .unwrap();
        assert_eq!(lobby.game, GameKind::TexasHoldem);
        assert!(lobby.texas_holdem_rules().unwrap().short_deck);
        assert!(lobby.texas_holdem_rules().unwrap().ignore_kickers);
        assert!(lobby.texas_holdem_rules().unwrap().omaha);
        assert_eq!(lobby.host_port, 52300);
        assert!(session.game().is_none());
    }

    #[test]
    fn lobby_disconnect_releases_the_player_from_count_and_capacity() {
        let mut session =
            TexasHoldemSession::new(ROOM, RuleSet::default(), build_deck(false)).unwrap();
        session.handle(HOST_CONNECTION, message(1, join_command("房主", 1)));
        session.handle(SECOND_CONNECTION, message(1, join_command("玩家二", 2)));
        session.handle(THIRD_CONNECTION, message(1, join_command("玩家三", 3)));

        let deliveries = session.disconnect(SECOND_CONNECTION);

        assert!(session.room.players[1].left);
        assert_eq!(
            session
                .room
                .players
                .iter()
                .filter(|player| !player.left)
                .count(),
            2
        );
        assert!(deliveries.iter().all(|delivery| {
            let ServerEvent::LobbySnapshot(snapshot) = &delivery.message.event else {
                return false;
            };
            snapshot.players.len() == 2 && snapshot.players.iter().all(|player| player.connected)
        }));

        let replacement = ConnectionId(40);
        let rejoined = session.handle(replacement, message(1, join_command("新玩家", 4)));
        assert!(rejoined.iter().any(|delivery| matches!(
            &delivery.message.event,
            ServerEvent::LobbySnapshot(snapshot) if snapshot.players.len() == 3
        )));
    }
}
