use std::collections::HashSet;
use std::time::Duration;

use leocard_protocol::{
    ClientCommand, ClientMessage, GameCommand, GameEvent, GameKind, GameRules, GameSnapshot,
    GameViolation, LobbySnapshot, PlayerId, PlayerInteraction, PlayerInteractionKind,
    PlayerReferenceChange, RejectReason, RequestId, Revision, RoomId, ServerEvent, UnoCommand,
    UnoEvent, UnoPhaseView, UnoPlayerResult, UnoPlayerState, UnoRevealedHand, UnoSnapshot,
    UnoViolation,
};
use leocard_uno::{
    ActionOutcome, Card, Color, GameError, GameState, Phase, PlayerId as CorePlayerId, RuleSet,
    build_deck,
};

use crate::{
    AUTO_PLAY_DELAY, AutoPlayDelayState, ConnectionId, Delivery, HostError, RoomSession,
    new_match_id,
};

#[derive(Clone, Debug)]
pub struct UnoSession {
    room: RoomSession,
    rules: RuleSet,
    shuffled_deck: Option<Vec<Card>>,
    game: Option<GameState>,
    match_id: Option<leocard_protocol::MatchId>,
    finished_reference_changes: Option<Vec<PlayerReferenceChange>>,
    auto_play_delay: Option<AutoPlayDelayState>,
}

impl UnoSession {
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
        let rules = rules.validate().map_err(GameError::from)?;
        validate_deck(&shuffled_deck)?;
        Ok(Self {
            room: RoomSession::new(room_id, host_port, usize::from(RuleSet::MAX_PLAYERS)),
            rules,
            shuffled_deck: Some(shuffled_deck),
            game: None,
            match_id: None,
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

    pub const fn game(&self) -> Option<&GameState> {
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

    pub fn advance_time(&mut self, elapsed: Duration) -> Vec<Delivery> {
        if elapsed.is_zero() || self.game.is_none() {
            return Vec::new();
        }
        let Some(player) = self.current_automatic_player() else {
            self.auto_play_delay = None;
            return Vec::new();
        };
        let disconnected = self
            .room
            .players
            .iter()
            .find(|participant| participant.id == player)
            .is_some_and(|participant| !participant.connected && !participant.is_bot);
        if !disconnected {
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
        }
        self.auto_play_delay = None;
        let Some(events) = self.play_automatic_action() else {
            return Vec::new();
        };
        self.apply_finished_reference_points();
        self.reset_auto_play_delay();
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
        self.room.players[index].connected = false;
        if self.room.host_connection == Some(connection) {
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
        if self.game.is_none() {
            self.room.players[index].seat = None;
            self.room.players[index].ready = false;
            self.room.players[index].left = true;
        }
        self.reset_auto_play_delay();
        self.room.bump_revision();
        if self.game.is_some() {
            self.broadcast_game(None)
        } else {
            self.broadcast_lobby(None)
        }
    }

    pub fn handle(&mut self, connection: ConnectionId, message: ClientMessage) -> Vec<Delivery> {
        if let Err(deliveries) = self.room.begin_request(connection, &message) {
            return deliveries;
        }
        self.apply_finished_reference_points();
        let request_id = message.request_id;
        match message.command {
            ClientCommand::Join {
                name,
                reconnect_token,
                profile_id,
                reference_points,
                completed_games,
                identity_signature,
            } => match self.room.join(
                connection,
                request_id,
                name,
                reconnect_token,
                profile_id,
                reference_points,
                completed_games,
                identity_signature,
                self.game.is_some(),
                RuleSet::MAX_PLAYERS,
            ) {
                Ok(mut deliveries) => {
                    deliveries.extend(if self.game.is_some() {
                        self.broadcast_game(Some((connection, request_id)))
                    } else {
                        self.broadcast_lobby(Some((connection, request_id)))
                    });
                    deliveries
                }
                Err(reason) => self.room.reject(connection, request_id, reason),
            },
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
            ClientCommand::Game(GameCommand::Uno(command)) => {
                self.handle_uno_command(connection, request_id, command)
            }
            ClientCommand::Game(command) => self.room.reject(
                connection,
                request_id,
                RejectReason::WrongGame {
                    expected: GameKind::Uno,
                    received: command.kind(),
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

    fn handle_uno_command(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        command: UnoCommand,
    ) -> Vec<Delivery> {
        match command {
            UnoCommand::SetAutoPlay { enabled } => {
                self.set_auto_play(connection, request_id, enabled)
            }
            UnoCommand::UpdateRules { rules } => self.update_rules(connection, request_id, rules),
            UnoCommand::ChooseInitialColor { color } => {
                self.perform_action(connection, request_id, Vec::new(), |game, player| {
                    game.choose_initial_color(player, color)
                })
            }
            UnoCommand::PlayCard { card, chosen_color } => self.perform_action(
                connection,
                request_id,
                vec![(card, chosen_color)],
                |game, player| game.play_card(player, card, chosen_color),
            ),
            UnoCommand::PlayCards {
                cards,
                chosen_color,
            } => {
                let played = cards
                    .iter()
                    .copied()
                    .map(|card| (card, chosen_color))
                    .collect();
                self.perform_action(connection, request_id, played, move |game, player| {
                    game.play_cards(player, &cards, chosen_color)
                })
            }
            UnoCommand::JumpIn { card } => self.perform_action(
                connection,
                request_id,
                vec![(card, None)],
                |game, player| game.jump_in(player, card),
            ),
            UnoCommand::DrawCard => {
                self.perform_action(connection, request_id, Vec::new(), GameState::draw_card)
            }
            UnoCommand::PassAfterDraw => self.perform_action(
                connection,
                request_id,
                Vec::new(),
                GameState::pass_after_draw,
            ),
            UnoCommand::AcceptDrawPenalty => self.perform_action(
                connection,
                request_id,
                Vec::new(),
                GameState::accept_draw_penalty,
            ),
            UnoCommand::ChallengeDrawFour => self.perform_action(
                connection,
                request_id,
                Vec::new(),
                GameState::challenge_draw_four,
            ),
            UnoCommand::ResolveSkip => {
                self.perform_action(connection, request_id, Vec::new(), GameState::resolve_skip)
            }
            UnoCommand::CallUno => {
                self.perform_action(connection, request_id, Vec::new(), GameState::call_uno)
            }
            UnoCommand::ReportUno { target } => {
                let core_target = to_core_player(target);
                self.perform_action(connection, request_id, Vec::new(), move |game, reporter| {
                    game.report_uno(reporter, core_target)
                })
            }
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
        let Ok(rules) = rules.validate() else {
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
            let mut deck = build_deck();
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
        if matches!(game.phase(), Phase::Finished(_)) {
            return self.reject_game_error(connection, request_id, &GameError::GameAlreadyFinished);
        }
        let participant = self
            .room
            .players
            .iter_mut()
            .find(|participant| participant.id == player)
            .expect("joined player belongs to room");
        if participant.auto_play != enabled {
            participant.auto_play = enabled;
            self.room.bump_revision();
        }
        self.reset_auto_play_delay();
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
        let active = self
            .room
            .players
            .iter()
            .filter(|player| !player.left)
            .count();
        if active < usize::from(RuleSet::MIN_PLAYERS) {
            return self.room.reject(
                connection,
                request_id,
                RejectReason::NotEnoughPlayers {
                    minimum: RuleSet::MIN_PLAYERS,
                    actual: active as u8,
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
            .filter(|player| !player.left && !player.ready)
            .map(|player| player.id)
            .collect::<Vec<_>>();
        if !not_ready.is_empty() {
            return self.room.reject(
                connection,
                request_id,
                RejectReason::PlayersNotReady { players: not_ready },
            );
        }
        self.room.remove_departed_players();
        self.room.players.sort_by_key(|player| {
            player
                .seat
                .expect("start validation required every player to select a seat")
                .0
        });
        for (index, player) in self.room.players.iter_mut().enumerate() {
            player.id = PlayerId(index as u8);
            player.ready = false;
            player.auto_play = player.is_bot;
        }
        let deck = self.shuffled_deck.take().unwrap_or_else(shuffled_uno_deck);
        match GameState::new_with_deck(self.rules, active as u8, deck) {
            Ok(game) => {
                self.game = Some(game);
                self.match_id = Some(new_match_id());
                self.finished_reference_changes = None;
                self.reset_auto_play_delay();
                self.room.bump_revision();
                self.broadcast_game(Some((connection, request_id)))
            }
            Err(_) => self.room.reject(
                connection,
                request_id,
                RejectReason::InvalidRuleConfiguration,
            ),
        }
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
        if self.room.host_connection != Some(connection) {
            return self.room.reject(
                connection,
                request_id,
                RejectReason::OnlyHostCanReturnToLobby,
            );
        }
        if !self
            .game
            .as_ref()
            .is_some_and(|game| matches!(game.phase(), Phase::Finished(_)))
        {
            return self
                .room
                .reject(connection, request_id, RejectReason::GameNotFinished);
        }
        self.game = None;
        self.match_id = None;
        self.finished_reference_changes = None;
        self.auto_play_delay = None;
        #[cfg(feature = "developer")]
        self.room.remove_developer_bots();
        let host = self.room.host_connection;
        for player in &mut self.room.players {
            player.ready = host == Some(player.connection);
            player.auto_play = false;
            if !player.connected {
                player.seat = None;
            }
        }
        self.shuffled_deck = Some(shuffled_uno_deck());
        self.room.bump_revision();
        self.broadcast_lobby(Some((connection, request_id)))
    }

    fn play_again(&mut self, connection: ConnectionId, request_id: RequestId) -> Vec<Delivery> {
        let Some(player) = self.room.player_id(connection) else {
            return self
                .room
                .reject(connection, request_id, RejectReason::NotJoined);
        };
        if !self
            .game
            .as_ref()
            .is_some_and(|game| matches!(game.phase(), Phase::Finished(_)))
        {
            return self
                .room
                .reject(connection, request_id, RejectReason::GameNotFinished);
        }
        if let Some(participant) = self.room.players.iter_mut().find(|item| item.id == player) {
            participant.ready = true;
        }
        self.room.bump_revision();
        let active = self.room.players.iter().filter(|player| !player.left);
        if active.clone().count() >= usize::from(RuleSet::MIN_PLAYERS)
            && active.clone().all(|player| player.ready)
        {
            return self.start_next_game(Some((connection, request_id)));
        }
        self.broadcast_game(Some((connection, request_id)))
    }

    fn start_next_game(&mut self, origin: Option<(ConnectionId, RequestId)>) -> Vec<Delivery> {
        self.room.remove_departed_players();
        for player in &mut self.room.players {
            player.ready = false;
            player.auto_play = player.is_bot;
        }
        let player_count = self.room.players.len() as u8;
        let game = GameState::new_with_deck(self.rules, player_count, shuffled_uno_deck())
            .expect("a freshly built UNO deck is valid");
        self.game = Some(game);
        self.match_id = Some(new_match_id());
        self.finished_reference_changes = None;
        self.reset_auto_play_delay();
        self.room.bump_revision();
        self.broadcast_game(origin)
    }

    fn perform_action<F>(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        played: Vec<(Card, Option<Color>)>,
        action: F,
    ) -> Vec<Delivery>
    where
        F: FnOnce(&mut GameState, CorePlayerId) -> Result<ActionOutcome, GameError>,
    {
        let Some(player) = self.room.player_id(connection) else {
            return self
                .room
                .reject(connection, request_id, RejectReason::NotJoined);
        };
        let Some(game) = self.game.as_mut() else {
            return self
                .room
                .reject(connection, request_id, RejectReason::GameNotStarted);
        };
        match action(game, to_core_player(player)) {
            Ok(outcome) => {
                let mut events = events_for_outcome(&outcome, &played);
                self.apply_finished_reference_points();
                self.reset_auto_play_delay();
                self.room.bump_revision();
                let mut deliveries = self.broadcast_events(std::mem::take(&mut events));
                deliveries.extend(self.broadcast_game(Some((connection, request_id))));
                deliveries
            }
            Err(error) => self.reject_game_error(connection, request_id, &error),
        }
    }

    fn apply_finished_reference_points(&mut self) {
        if self.finished_reference_changes.is_some() {
            return;
        }
        let Some(game) = self.game.as_ref() else {
            return;
        };
        let Phase::Finished(result) = game.phase() else {
            return;
        };
        self.room.prepare_rematch();
        let mut changes = Vec::with_capacity(result.reference_deltas.len());
        for (index, delta) in result.reference_deltas.iter().copied().enumerate() {
            let participant = self
                .room
                .players
                .iter_mut()
                .find(|player| player.id == PlayerId(index as u8))
                .expect("a core UNO player belongs to the room");
            participant.reference_points = participant
                .reference_points
                .saturating_add(i32::from(delta));
            participant.completed_games = participant.completed_games.saturating_add(1);
            changes.push(PlayerReferenceChange {
                player: participant.id,
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
        let name = self.room.players[index].name.clone();
        self.room.players[index].ready = false;
        self.room.players[index].connected = false;
        self.room.players[index].left = true;
        if self.game.is_none() {
            self.room.players[index].seat = None;
        }
        self.reset_auto_play_delay();
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
        if !self
            .game
            .as_ref()
            .is_some_and(|game| matches!(game.phase(), Phase::Playing))
        {
            return self
                .room
                .reject(connection, request_id, RejectReason::GameNotStarted);
        }
        if source == target
            || !self
                .room
                .players
                .iter()
                .any(|player| player.id == target && !player.left)
        {
            return self.reject_game_error(
                connection,
                request_id,
                &GameError::InvalidPlayer(to_core_player(target)),
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
            ServerEvent::GameSnapshot(GameSnapshot::Uno(self.game_snapshot(player)))
        } else {
            ServerEvent::LobbySnapshot(self.lobby_snapshot())
        };
        vec![self.room.delivery(connection, Some(request_id), event)]
    }

    fn current_automatic_player(&self) -> Option<PlayerId> {
        let current = self
            .game
            .as_ref()?
            .turn()
            .map(|turn| from_core_player(turn.current_player))?;
        self.room
            .players
            .iter()
            .find(|player| player.id == current && !player.left)
            .is_some_and(|player| player.auto_play || !player.connected || player.is_bot)
            .then_some(current)
    }

    fn reset_auto_play_delay(&mut self) {
        self.auto_play_delay = self
            .current_automatic_player()
            .map(|player| AutoPlayDelayState {
                player,
                remaining: AUTO_PLAY_DELAY,
            });
    }

    fn play_automatic_action(&mut self) -> Option<Vec<UnoEvent>> {
        let game = self.game.as_mut()?;
        let turn = game.turn()?;
        let player = turn.current_player;
        let mut events = game
            .call_uno(player)
            .ok()
            .map(|outcome| events_for_outcome(&outcome, &[]))
            .unwrap_or_default();
        let mut played = Vec::new();
        let outcome = if turn.current_color.is_none() {
            game.choose_initial_color(player, preferred_color(game, player))
        } else if turn.pending_skip > 0 || turn.skipped_turns_remaining > 0 {
            if turn.skipped_turns_remaining == 0
                && let Some(card) = game
                    .player(player)?
                    .hand()
                    .iter()
                    .copied()
                    .find(|card| game.can_play(player, *card))
            {
                played.push((card, None));
                game.play_card(player, card, None)
            } else {
                game.resolve_skip(player)
            }
        } else if let Some(card) = turn.drawn_card {
            let color = card.face().is_wild().then(|| preferred_color(game, player));
            played.push((card, color));
            game.play_card(player, card, color)
        } else if turn.pending_draw > 0 {
            if let Some(card) = game
                .player(player)?
                .hand()
                .iter()
                .copied()
                .find(|card| game.can_play(player, *card))
            {
                let color = card.face().is_wild().then(|| preferred_color(game, player));
                played.push((card, color));
                game.play_card(player, card, color)
            } else {
                game.accept_draw_penalty(player)
            }
        } else if let Some(card) = game
            .player(player)?
            .hand()
            .iter()
            .copied()
            .find(|card| game.can_play(player, *card))
        {
            let color = card.face().is_wild().then(|| preferred_color(game, player));
            played.push((card, color));
            game.play_card(player, card, color)
        } else {
            game.draw_card(player)
        }
        .ok()?;
        events.extend(events_for_outcome(&outcome, &played));
        Some(events)
    }

    fn lobby_snapshot(&self) -> LobbySnapshot {
        self.room
            .lobby_snapshot(GameKind::Uno, GameRules::Uno(self.rules))
    }

    fn broadcast_lobby(&self, origin: Option<(ConnectionId, RequestId)>) -> Vec<Delivery> {
        self.room
            .broadcast_lobby(GameKind::Uno, GameRules::Uno(self.rules), origin)
    }

    fn broadcast_game(&self, origin: Option<(ConnectionId, RequestId)>) -> Vec<Delivery> {
        assert!(self.game.is_some(), "game broadcast requires an UNO game");
        self.room
            .players
            .iter()
            .filter(|player| player.connected && !player.left)
            .map(|player| {
                let reply = origin
                    .filter(|(connection, _)| *connection == player.connection)
                    .map(|(_, request)| request);
                self.room.delivery(
                    player.connection,
                    reply,
                    ServerEvent::GameSnapshot(GameSnapshot::Uno(self.game_snapshot(player.id))),
                )
            })
            .collect()
    }

    fn broadcast_events(&self, events: Vec<UnoEvent>) -> Vec<Delivery> {
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
                            ServerEvent::GameEvent(GameEvent::Uno(event.clone())),
                        )
                    })
            })
            .collect()
    }

    fn game_snapshot(&self, recipient: PlayerId) -> UnoSnapshot {
        let game = self.game.as_ref().expect("an UNO snapshot requires a game");
        let turn = game.turn();
        let players = self
            .room
            .players
            .iter()
            .filter(|player| !player.left)
            .map(|participant| {
                let state = game
                    .player(to_core_player(participant.id))
                    .expect("room and UNO core players stay aligned");
                UnoPlayerState {
                    id: participant.id,
                    profile_id: participant.profile_id,
                    name: participant.name.clone(),
                    avatar: participant.avatar,
                    seat: participant.seat.expect("started players retain seats"),
                    hand_len: state.hand().len() as u8,
                    ready: participant.ready,
                    connected: (participant.connected || participant.is_bot) && !participant.left,
                    auto_play: participant.auto_play,
                    reference_points: participant.reference_points,
                    completed_games: participant.completed_games,
                    skipped_turns: game
                        .skipped_turns(to_core_player(participant.id))
                        .unwrap_or(0),
                }
            })
            .collect();
        let phase = match game.phase() {
            Phase::Playing => UnoPhaseView::Playing,
            Phase::Finished(result) => UnoPhaseView::Finished {
                winner: from_core_player(result.winner),
                results: result
                    .hand_scores
                    .iter()
                    .enumerate()
                    .map(|(index, hand_score)| UnoPlayerResult {
                        player: PlayerId(index as u8),
                        hand_score: *hand_score,
                        placement: result.placements[index],
                    })
                    .collect(),
                remaining_hands: game
                    .players()
                    .iter()
                    .map(|player| UnoRevealedHand {
                        player: from_core_player(player.id()),
                        cards: player.hand().to_vec(),
                    })
                    .collect(),
                reference_changes: self
                    .finished_reference_changes
                    .clone()
                    .expect("UNO points are applied before final snapshot"),
            },
        };
        UnoSnapshot {
            match_id: self.match_id.expect("a running UNO game has a match id"),
            host_port: self.room.host_port,
            you: recipient,
            host: self
                .room
                .host_player_id()
                .expect("a running room has a host"),
            rules: self.rules,
            players,
            your_hand: game
                .player(to_core_player(recipient))
                .expect("recipient belongs to game")
                .hand()
                .to_vec(),
            draw_pile_len: game.draw_pile_len() as u16,
            discard_top: game.top_card(),
            discard_pile: game
                .discard_pile()
                .iter()
                .rev()
                .take(6)
                .copied()
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect(),
            current_color: game.current_color(),
            current_player: turn.map(|turn| from_core_player(turn.current_player)),
            direction: game.direction(),
            pending_draw: turn.map_or(0, |turn| turn.pending_draw),
            pending_kind: turn.and_then(|turn| turn.pending_kind),
            challenge_offender: turn
                .and_then(|turn| turn.challenge_offender)
                .map(from_core_player),
            pending_skip: turn.map_or(0, |turn| turn.pending_skip),
            your_drawn_card: turn
                .filter(|turn| from_core_player(turn.current_player) == recipient)
                .and_then(|turn| turn.drawn_card),
            your_jump_in_card: game.jump_in_card(to_core_player(recipient)),
            uno_exposed: game.uno_exposed_players().map(from_core_player).collect(),
            uno_declared: game.uno_declared_players().map(from_core_player).collect(),
            phase,
        }
    }

    fn reject_game_error(
        &self,
        connection: ConnectionId,
        request_id: RequestId,
        error: &GameError,
    ) -> Vec<Delivery> {
        self.room.reject(
            connection,
            request_id,
            RejectReason::GameViolation(GameViolation::Uno(map_game_error(error))),
        )
    }
}

fn events_for_outcome(outcome: &ActionOutcome, played: &[(Card, Option<Color>)]) -> Vec<UnoEvent> {
    let mut events = played
        .iter()
        .copied()
        .map(|(card, chosen_color)| match outcome {
            ActionOutcome::Played { player, .. } => UnoEvent::CardPlayed {
                player: from_core_player(*player),
                card,
                chosen_color,
            },
            ActionOutcome::GameFinished(result) => UnoEvent::CardPlayed {
                player: from_core_player(result.winner),
                card,
                chosen_color,
            },
            _ => unreachable!("only play actions carry played-card metadata"),
        })
        .collect::<Vec<_>>();
    match outcome {
        ActionOutcome::ColorChosen { player, color } => events.push(UnoEvent::ColorChosen {
            player: from_core_player(*player),
            color: *color,
        }),
        ActionOutcome::DrewCard { player, .. } => events.push(UnoEvent::CardsDrawn {
            player: from_core_player(*player),
            count: 1,
            penalty: false,
        }),
        ActionOutcome::PenaltyDrawn { player, cards, .. } => {
            events.push(UnoEvent::CardsDrawn {
                player: from_core_player(*player),
                count: cards.len() as u16,
                penalty: true,
            });
        }
        ActionOutcome::ChallengeResolved {
            challenger,
            offender,
            result,
            penalized,
            cards,
            ..
        } => events.push(UnoEvent::ChallengeResolved {
            challenger: from_core_player(*challenger),
            offender: from_core_player(*offender),
            result: *result,
            penalized: from_core_player(*penalized),
            count: cards.len() as u16,
        }),
        ActionOutcome::UnoCalled { player } => events.push(UnoEvent::UnoCalled {
            player: from_core_player(*player),
        }),
        ActionOutcome::UnoReported {
            reporter, target, ..
        } => events.push(UnoEvent::UnoReported {
            reporter: from_core_player(*reporter),
            target: from_core_player(*target),
        }),
        ActionOutcome::SkipResolved {
            player,
            cards,
            remaining,
            ..
        } => events.push(UnoEvent::SkipResolved {
            player: from_core_player(*player),
            remaining: *remaining,
            drew_card: !cards.is_empty(),
        }),
        ActionOutcome::GameFinished(result) => events.push(UnoEvent::GameFinished {
            winner: from_core_player(result.winner),
        }),
        ActionOutcome::Played { .. } | ActionOutcome::PassedAfterDraw { .. } => {}
    }
    events
}

fn preferred_color(game: &GameState, player: CorePlayerId) -> Color {
    let mut counts = [0_u8; 4];
    if let Some(state) = game.player(player) {
        for color in state.hand().iter().filter_map(|card| card.color()) {
            let index = match color {
                Color::Red => 0,
                Color::Yellow => 1,
                Color::Green => 2,
                Color::Blue => 3,
            };
            counts[index] = counts[index].saturating_add(1);
        }
    }
    Color::ALL
        .into_iter()
        .enumerate()
        .max_by_key(|(index, _)| (counts[*index], std::cmp::Reverse(*index)))
        .map(|(_, color)| color)
        .unwrap_or(Color::Red)
}

fn shuffled_uno_deck() -> Vec<Card> {
    let mut deck = build_deck();
    fastrand::shuffle(&mut deck);
    deck
}

fn validate_deck(deck: &[Card]) -> Result<(), HostError> {
    let expected_deck = build_deck();
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

fn to_core_player(player: PlayerId) -> CorePlayerId {
    CorePlayerId(usize::from(player.0))
}

fn from_core_player(player: CorePlayerId) -> PlayerId {
    PlayerId(u8::try_from(player.0).expect("UNO supports at most six players"))
}

fn map_game_error(error: &GameError) -> UnoViolation {
    match error {
        GameError::InvalidPlayer(_) => UnoViolation::InvalidPlayer,
        GameError::NotPlayersTurn { .. } => UnoViolation::NotPlayersTurn,
        GameError::GameAlreadyFinished => UnoViolation::GameAlreadyFinished,
        GameError::InitialColorChoiceRequired => UnoViolation::InitialColorChoiceRequired,
        GameError::InitialColorAlreadyChosen => UnoViolation::InitialColorAlreadyChosen,
        GameError::CardNotInHand(_) => UnoViolation::CardNotInHand,
        GameError::CardDoesNotMatch => UnoViolation::CardDoesNotMatch,
        GameError::ColorRequired => UnoViolation::ColorRequired,
        GameError::UnexpectedColor => UnoViolation::UnexpectedColor,
        GameError::MustPlayDrawnCard(_) => UnoViolation::MustPlayDrawnCard,
        GameError::MustResolveDrawPenalty => UnoViolation::MustResolveDrawPenalty,
        GameError::NoDrawPenalty => UnoViolation::NoDrawPenalty,
        GameError::CannotStack(_) => UnoViolation::CannotStack,
        GameError::CannotChallenge => UnoViolation::CannotChallenge,
        GameError::MustDrawBeforePassing => UnoViolation::MustDrawBeforePassing,
        GameError::MustResolveSkip => UnoViolation::MustResolveSkip,
        GameError::NoSkipToResolve => UnoViolation::NoSkipToResolve,
        GameError::UnoCalloutDisabled => UnoViolation::UnoCalloutDisabled,
        GameError::CannotCallUno(_) => UnoViolation::CannotCallUno,
        GameError::MustPlayAfterUno => UnoViolation::MustPlayAfterUno,
        GameError::CannotReportSelf => UnoViolation::CannotReportSelf,
        GameError::PlayerNotReportable(_) => UnoViolation::PlayerNotReportable,
        GameError::CannotPlayTogether => UnoViolation::CannotPlayTogether,
        GameError::CannotJumpIn => UnoViolation::CannotJumpIn,
        GameError::DrawPileExhausted => UnoViolation::DrawPileExhausted,
        GameError::InvalidRules(_)
        | GameError::InvalidDeckSize { .. }
        | GameError::InvalidDeckContents => UnoViolation::CardDoesNotMatch,
    }
}

#[cfg(test)]
mod tests {
    use ed25519_dalek::{Signer, SigningKey};
    use leocard_protocol::{ProfileId, ReconnectToken, SeatId, join_identity_payload};

    use super::*;

    const ROOM: RoomId = RoomId(108);
    const HOST: ConnectionId = ConnectionId(1);

    fn message(request: u64, command: ClientCommand) -> ClientMessage {
        ClientMessage::new(ROOM, RequestId(request), command)
    }

    fn join_command(name: &str, token: u64) -> ClientCommand {
        let mut secret = [0; 32];
        secret[..8].copy_from_slice(&token.to_be_bytes());
        secret[8] = 7;
        let key = SigningKey::from_bytes(&secret);
        let reconnect_token = ReconnectToken(token);
        let payload = join_identity_payload(ROOM, reconnect_token, name, 0, 0);
        ClientCommand::Join {
            name: name.to_owned(),
            reconnect_token,
            profile_id: ProfileId(key.verifying_key().to_bytes()),
            reference_points: 0,
            completed_games: 0,
            identity_signature: key.sign(&payload).to_bytes().to_vec(),
        }
    }

    #[test]
    fn two_players_can_start_and_receive_private_hands() {
        let mut session = UnoSession::new(ROOM, RuleSet::default(), build_deck()).unwrap();
        session.handle(HOST, message(1, join_command("甲", 1)));
        let second = ConnectionId(2);
        session.handle(second, message(1, join_command("乙", 2)));
        session.handle(second, message(2, ClientCommand::SetReady { ready: true }));
        let deliveries = session.handle(HOST, message(2, ClientCommand::StartGame));
        let snapshots = deliveries
            .iter()
            .filter_map(|delivery| match &delivery.message.event {
                ServerEvent::GameSnapshot(GameSnapshot::Uno(snapshot)) => Some(snapshot),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(snapshots.len(), 2);
        assert!(
            snapshots
                .iter()
                .all(|snapshot| snapshot.your_hand.len() == 7)
        );
        assert!(snapshots.iter().all(|snapshot| snapshot.players.len() == 2));
    }

    #[test]
    fn wrong_game_command_is_rejected_with_uno_as_expected_kind() {
        let mut session = UnoSession::new(ROOM, RuleSet::default(), build_deck()).unwrap();
        session.handle(HOST, message(1, join_command("甲", 1)));
        let deliveries = session.handle(
            HOST,
            message(
                2,
                ClientCommand::Game(GameCommand::TexasHoldem(
                    leocard_protocol::TexasHoldemCommand::SetAutoPlay { enabled: true },
                )),
            ),
        );
        assert!(matches!(
            deliveries[0].message.event,
            ServerEvent::Rejected {
                reason: RejectReason::WrongGame {
                    expected: GameKind::Uno,
                    received: GameKind::TexasHoldem,
                }
            }
        ));
    }

    #[test]
    fn uno_room_capacity_is_always_six() {
        let mut session = UnoSession::new(ROOM, RuleSet::default(), build_deck()).unwrap();
        session.handle(HOST, message(1, join_command("P0", 10)));
        for index in 1..6_u64 {
            let deliveries = session.handle(
                ConnectionId(index + 1),
                message(1, join_command(&format!("P{index}"), index + 10)),
            );
            assert!(deliveries.iter().all(|delivery| {
                !matches!(
                    delivery.message.event,
                    ServerEvent::Rejected {
                        reason: RejectReason::RoomFull
                    }
                )
            }));
        }
        assert_eq!(session.room.players.len(), 6);
    }

    fn jump_in_session() -> (UnoSession, ConnectionId, ConnectionId, Card, Card) {
        let first = Card::number(Color::Red, 7, 0);
        let matching = Card::number(Color::Red, 7, 1);
        let start = Card::number(Color::Red, 5, 0);
        let mut deck = build_deck();
        for (position, required) in [(0, first), (2, matching), (21, start)] {
            let current = deck
                .iter()
                .position(|candidate| *candidate == required)
                .unwrap();
            deck.swap(position, current);
        }
        let mut session = UnoSession::new(
            ROOM,
            RuleSet {
                stack_skip: true,
                jump_in: true,
                ..RuleSet::default()
            },
            deck,
        )
        .unwrap();
        let second = ConnectionId(2);
        let third = ConnectionId(3);
        session.handle(HOST, message(1, join_command("甲", 1)));
        session.handle(second, message(1, join_command("乙", 2)));
        session.handle(third, message(1, join_command("丙", 3)));
        for (index, player) in session.room.players.iter_mut().enumerate() {
            player.seat = Some(SeatId(index as u8));
        }
        session.handle(second, message(2, ClientCommand::SetReady { ready: true }));
        session.handle(third, message(2, ClientCommand::SetReady { ready: true }));
        session.handle(HOST, message(2, ClientCommand::StartGame));
        (session, second, third, first, matching)
    }

    fn uno_snapshot_for(deliveries: &[Delivery], recipient: ConnectionId) -> &UnoSnapshot {
        deliveries
            .iter()
            .find_map(|delivery| {
                (delivery.recipient == recipient)
                    .then_some(&delivery.message.event)
                    .and_then(|event| match event {
                        ServerEvent::GameSnapshot(GameSnapshot::Uno(snapshot)) => Some(snapshot),
                        _ => None,
                    })
            })
            .expect("recipient should receive an UNO snapshot")
    }

    #[test]
    fn jump_in_candidate_is_private_and_successful_command_moves_play_to_that_player() {
        let (mut session, second, third, first, matching) = jump_in_session();
        let deliveries = session.handle(
            HOST,
            message(
                3,
                ClientCommand::Game(GameCommand::Uno(UnoCommand::PlayCard {
                    card: first,
                    chosen_color: None,
                })),
            ),
        );
        assert_eq!(
            uno_snapshot_for(&deliveries, second).your_jump_in_card,
            None
        );
        assert_eq!(
            uno_snapshot_for(&deliveries, third).your_jump_in_card,
            Some(matching)
        );

        let deliveries = session.handle(
            third,
            message(
                3,
                ClientCommand::Game(GameCommand::Uno(UnoCommand::JumpIn { card: matching })),
            ),
        );
        let snapshot = uno_snapshot_for(&deliveries, third);
        assert_eq!(snapshot.discard_top, matching);
        assert_eq!(snapshot.current_player, Some(PlayerId(0)));
        assert_eq!(snapshot.your_hand.len(), 6);
    }

    #[test]
    fn any_successful_next_player_action_closes_server_jump_in_window() {
        let (mut session, second, third, first, matching) = jump_in_session();
        session.handle(
            HOST,
            message(
                3,
                ClientCommand::Game(GameCommand::Uno(UnoCommand::PlayCard {
                    card: first,
                    chosen_color: None,
                })),
            ),
        );
        session.handle(
            second,
            message(
                3,
                ClientCommand::Game(GameCommand::Uno(UnoCommand::DrawCard)),
            ),
        );
        let deliveries = session.handle(
            third,
            message(
                3,
                ClientCommand::Game(GameCommand::Uno(UnoCommand::JumpIn { card: matching })),
            ),
        );
        assert!(deliveries.iter().any(|delivery| {
            delivery.recipient == third
                && matches!(
                    delivery.message.event,
                    ServerEvent::Rejected {
                        reason: RejectReason::GameViolation(GameViolation::Uno(
                            UnoViolation::CannotJumpIn
                        ))
                    }
                )
        }));
    }

    #[test]
    fn identical_pair_broadcasts_both_card_play_events_in_order() {
        let first = Card::number(Color::Red, 7, 0);
        let second = Card::number(Color::Red, 7, 1);
        let events = events_for_outcome(
            &ActionOutcome::Played {
                player: CorePlayerId(0),
                card: second,
                next_player: CorePlayerId(1),
            },
            &[(first, None), (second, None)],
        );
        assert_eq!(
            events,
            vec![
                UnoEvent::CardPlayed {
                    player: PlayerId(0),
                    card: first,
                    chosen_color: None,
                },
                UnoEvent::CardPlayed {
                    player: PlayerId(0),
                    card: second,
                    chosen_color: None,
                },
            ]
        );
    }
}
