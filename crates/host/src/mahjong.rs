use std::collections::HashSet;
use std::time::Duration;

use leocard_mahjong::{
    ActionOutcome, Claim, ClaimOption, GameError, GameState, MeldKind, Phase,
    PlayerId as CorePlayerId, RuleSet, Tile, TileKind, build_deck,
};
use leocard_protocol::{
    ClientCommand, ClientMessage, GameCommand, GameEvent, GameKind, GameRules, GameSnapshot,
    GameViolation, LobbySnapshot, MahjongCommand, MahjongDiscardView, MahjongEvent,
    MahjongHandResultView, MahjongPendingClaimView, MahjongPhaseView, MahjongPlayerState,
    MahjongPublicMeldView, MahjongSnapshot, MahjongViolation, MahjongWinView, PlayerId,
    PlayerInteraction, PlayerInteractionKind, RejectReason, RequestId, Revision, RoomId,
    ServerEvent,
};

use crate::{
    AUTO_PLAY_DELAY, AutoPlayDelayState, ConnectionId, Delivery, HostError, RoomSession,
    new_match_id,
};

const MAHJONG_DEAL_INTERVAL: Duration = Duration::from_millis(320);

#[derive(Clone, Copy, Debug)]
enum AutomaticMahjongAction {
    Discard(Tile),
    Respond(Claim),
    SelfDraw,
    ConcealedKong(TileKind),
    AddedKong(Tile),
}

#[derive(Clone, Debug)]
pub struct MahjongSession {
    room: RoomSession,
    rules: RuleSet,
    shuffled_deck: Option<Vec<Tile>>,
    game: Option<GameState>,
    match_id: Option<leocard_protocol::MatchId>,
    auto_play_delay: Option<AutoPlayDelayState>,
    deal_delay: Duration,
}

impl MahjongSession {
    pub fn new(
        room_id: RoomId,
        rules: RuleSet,
        shuffled_deck: Vec<Tile>,
    ) -> Result<Self, HostError> {
        Self::new_with_host_port(room_id, 52300, rules, shuffled_deck)
    }

    pub fn new_with_host_port(
        room_id: RoomId,
        host_port: u16,
        rules: RuleSet,
        shuffled_deck: Vec<Tile>,
    ) -> Result<Self, HostError> {
        let rules = rules.validate().map_err(GameError::from)?;
        validate_deck(&shuffled_deck)?;
        Ok(Self {
            room: RoomSession::new_with_seat_count(
                room_id,
                host_port,
                RuleSet::PLAYER_COUNT,
                RuleSet::PLAYER_COUNT as u8,
            ),
            rules,
            shuffled_deck: Some(shuffled_deck),
            game: None,
            match_id: None,
            auto_play_delay: None,
            deal_delay: Duration::ZERO,
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

    pub fn is_current_connection(&self, connection: ConnectionId) -> bool {
        self.room.player_id(connection).is_some()
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
        if self.game.as_ref().is_some_and(|game| {
            matches!(
                game.phase(),
                Phase::Dealing { .. } | Phase::ReplacingFlower { .. }
            )
        }) {
            if elapsed < self.deal_delay {
                self.deal_delay -= elapsed;
                return Vec::new();
            }
            self.deal_delay = MAHJONG_DEAL_INTERVAL;
            let game = self.game.as_mut().expect("checked Mahjong game exists");
            let flower_counts = game
                .players()
                .iter()
                .map(|player| player.flowers().len())
                .collect::<Vec<_>>();
            let advanced = match game.phase() {
                Phase::Dealing { .. } => game.advance_deal().map(|_| ()),
                Phase::ReplacingFlower { .. } => game.advance_flower_replacement().map(|_| ()),
                _ => unreachable!("checked timed Mahjong phase"),
            };
            if advanced.is_err() {
                return Vec::new();
            }
            let replaced = game
                .players()
                .iter()
                .enumerate()
                .find(|(index, player)| player.flowers().len() > flower_counts[*index])
                .map(|(index, _)| PlayerId(index as u8));
            self.room.bump_revision();
            let mut deliveries = replaced.map_or_else(Vec::new, |player| {
                self.broadcast_events(vec![MahjongEvent::FlowerReplaced { player }])
            });
            deliveries.extend(self.broadcast_game(None));
            return deliveries;
        }
        let Some(player) = self.current_automatic_player() else {
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
        let Some((public_event, outcome)) = self.play_automatic_action(player) else {
            return Vec::new();
        };
        if matches!(
            self.game.as_ref().map(GameState::phase),
            Some(Phase::Finished(_))
        ) {
            self.room.prepare_rematch();
        }
        if matches!(
            self.game.as_ref().map(GameState::phase),
            Some(Phase::ReplacingFlower { .. })
        ) {
            self.deal_delay = MAHJONG_DEAL_INTERVAL;
        }
        self.room.bump_revision();
        let mut events = public_event.into_iter().collect::<Vec<_>>();
        events.extend(events_for_outcome(&outcome));
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
            } => match self.room.join(
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
                RuleSet::PLAYER_COUNT as u8,
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
            ClientCommand::ConfigureBotSeat { seat, occupied } => match self
                .room
                .configure_bot_seat(connection, seat, occupied, self.game.is_some())
            {
                Ok(()) => self.broadcast_lobby(Some((connection, request_id))),
                Err(reason) => self.room.reject(connection, request_id, reason),
            },
            ClientCommand::SetReady { ready } => {
                match self.room.set_ready(connection, ready, self.game.is_some()) {
                    Ok(()) => self.broadcast_lobby(Some((connection, request_id))),
                    Err(reason) => self.room.reject(connection, request_id, reason),
                }
            }
            ClientCommand::Game(GameCommand::Mahjong(command)) => {
                self.handle_game_command(connection, request_id, command)
            }
            ClientCommand::Game(command) => self.room.reject(
                connection,
                request_id,
                RejectReason::WrongGame {
                    expected: GameKind::Mahjong,
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
            ClientCommand::Ping => unreachable!("transport pings are handled by HostSession"),
        }
    }

    fn handle_game_command(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        command: MahjongCommand,
    ) -> Vec<Delivery> {
        match command {
            MahjongCommand::UpdateRules { rules } => {
                self.update_rules(connection, request_id, rules)
            }
            MahjongCommand::Discard { tile } => self.perform_action(
                connection,
                request_id,
                Some(MahjongEvent::TileDiscarded {
                    player: self.room.player_id(connection).unwrap_or(PlayerId(u8::MAX)),
                    tile,
                }),
                move |game, player| game.discard(player, tile),
            ),
            MahjongCommand::RespondToClaim { claim } => {
                self.perform_action(connection, request_id, None, move |game, player| {
                    game.respond_to_claim(player, claim)
                })
            }
            MahjongCommand::DeclareSelfDraw => {
                self.perform_action(connection, request_id, None, GameState::declare_self_draw)
            }
            MahjongCommand::DeclareConcealedKong { tile } => {
                self.perform_action(connection, request_id, None, move |game, player| {
                    game.declare_concealed_kong(player, tile)
                })
            }
            MahjongCommand::DeclareAddedKong { tile } => {
                self.perform_action(connection, request_id, None, move |game, player| {
                    game.declare_added_kong(player, tile)
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
            self.room.reset_ready_after_rules_change();
            self.shuffled_deck = Some(shuffled_deck());
            self.room.bump_revision();
        }
        self.broadcast_lobby(Some((connection, request_id)))
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
        if active != RuleSet::PLAYER_COUNT {
            return self.room.reject(
                connection,
                request_id,
                RejectReason::WaitingForPlayers {
                    expected: RuleSet::PLAYER_COUNT as u8,
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
        }
        let deck = self.shuffled_deck.take().unwrap_or_else(shuffled_deck);
        match GameState::new_with_deck(self.rules, deck, CorePlayerId(0)) {
            Ok(game) => {
                self.game = Some(game);
                self.match_id = Some(new_match_id());
                self.deal_delay = Duration::ZERO;
                self.room.bump_revision();
                self.broadcast_game(Some((connection, request_id)))
            }
            Err(error) => self.reject_game_error(connection, request_id, &error),
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
        #[cfg(feature = "developer")]
        self.room.remove_developer_bots();
        let host = self.room.host_connection;
        for player in &mut self.room.players {
            player.ready = player.is_bot || host == Some(player.connection);
            player.auto_play = player.is_bot;
            if !player.connected {
                player.seat = None;
            }
        }
        self.shuffled_deck = Some(shuffled_deck());
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
        if !matches!(game.phase(), Phase::Finished(_)) {
            return self
                .room
                .reject(connection, request_id, RejectReason::GameNotFinished);
        }
        if let Some(participant) = self.room.players.iter_mut().find(|item| item.id == player) {
            participant.ready = true;
        }
        self.room.bump_revision();
        let mut active = self.room.players.iter().filter(|player| !player.left);
        if active.clone().count() == RuleSet::PLAYER_COUNT && active.all(|player| player.ready) {
            return self.start_next_hand(Some((connection, request_id)));
        }
        self.broadcast_game(Some((connection, request_id)))
    }

    fn start_next_hand(&mut self, origin: Option<(ConnectionId, RequestId)>) -> Vec<Delivery> {
        let completed_match = self.game.as_ref().is_some_and(
            |game| matches!(game.phase(), Phase::Finished(result) if result.match_complete),
        );
        let game = self.game.as_mut().expect("rematch requires an active game");
        game.start_next_hand(shuffled_deck())
            .expect("a freshly built Mahjong wall is valid");
        self.deal_delay = Duration::ZERO;
        if completed_match {
            self.match_id = Some(new_match_id());
        }
        for player in &mut self.room.players {
            player.ready = false;
        }
        self.room.bump_revision();
        self.broadcast_game(origin)
    }

    fn perform_action<F>(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        public_event: Option<MahjongEvent>,
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
        let dead_before = game
            .players()
            .iter()
            .map(|player| player.is_dead_hand())
            .collect::<Vec<_>>();
        match action(game, to_core_player(player)) {
            Ok(outcome) => {
                let mut events = Vec::new();
                if let Some(event) = public_event {
                    events.push(event);
                }
                events.extend(events_for_outcome(&outcome));
                for (index, core_player) in game.players().iter().enumerate() {
                    let already_reported = matches!(
                        outcome,
                        ActionOutcome::FalseWin { player, .. } if player.0 == index
                    );
                    if !already_reported && !dead_before[index] && core_player.is_dead_hand() {
                        let mut deltas = [10; RuleSet::PLAYER_COUNT];
                        deltas[index] = -30;
                        events.push(MahjongEvent::FalseWin {
                            player: PlayerId(index as u8),
                            deltas,
                        });
                    }
                }
                if matches!(game.phase(), Phase::Finished(_)) {
                    self.room.prepare_rematch();
                }
                if matches!(game.phase(), Phase::ReplacingFlower { .. }) {
                    self.deal_delay = MAHJONG_DEAL_INTERVAL;
                }
                self.room.bump_revision();
                let mut deliveries = self.broadcast_events(events);
                deliveries.extend(self.broadcast_game(Some((connection, request_id))));
                deliveries
            }
            Err(error) => self.reject_game_error(connection, request_id, &error),
        }
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
        &mut self,
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
        if self.game.is_none() {
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
            return self.room.reject(
                connection,
                request_id,
                RejectReason::GameViolation(GameViolation::Mahjong(
                    MahjongViolation::InvalidPlayer,
                )),
            );
        }
        let interaction = PlayerInteraction {
            source,
            target,
            kind,
            seed: fastrand::u32(..),
        };
        self.room.record_received_interaction(target, kind);
        self.room.bump_revision();
        let mut deliveries = self
            .room
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
            .collect::<Vec<_>>();
        deliveries.extend(self.broadcast_game(None));
        deliveries
    }

    fn snapshot(&self, connection: ConnectionId, request_id: RequestId) -> Vec<Delivery> {
        let Some(player) = self.room.player_id(connection) else {
            return self
                .room
                .reject(connection, request_id, RejectReason::NotJoined);
        };
        let event = if self.game.is_some() {
            ServerEvent::GameSnapshot(GameSnapshot::Mahjong(self.game_snapshot(player)))
        } else {
            ServerEvent::LobbySnapshot(self.lobby_snapshot())
        };
        vec![self.room.delivery(connection, Some(request_id), event)]
    }

    fn current_automatic_player(&self) -> Option<PlayerId> {
        let game = self.game.as_ref()?;
        let automatic = |player: CorePlayerId| {
            let player = from_core_player(player);
            self.room
                .players
                .iter()
                .find(|participant| participant.id == player && !participant.left)
                .is_some_and(|participant| participant.is_bot || participant.auto_play)
                .then_some(player)
        };
        match game.phase() {
            Phase::Playing => automatic(game.current_player()),
            Phase::WaitingForClaims(pending) => {
                pending.waiting_for().into_iter().find_map(automatic)
            }
            Phase::Dealing { .. } | Phase::ReplacingFlower { .. } | Phase::Finished(_) => None,
        }
    }

    fn play_automatic_action(
        &mut self,
        player: PlayerId,
    ) -> Option<(Option<MahjongEvent>, ActionOutcome)> {
        let core_player = to_core_player(player);
        let action = automatic_mahjong_action(self.game.as_ref()?, core_player)?;
        let game = self.game.as_mut()?;
        let public_event = match action {
            AutomaticMahjongAction::Discard(tile) => {
                Some(MahjongEvent::TileDiscarded { player, tile })
            }
            _ => None,
        };
        let outcome = match action {
            AutomaticMahjongAction::Discard(tile) => game.discard(core_player, tile),
            AutomaticMahjongAction::Respond(claim) => game.respond_to_claim(core_player, claim),
            AutomaticMahjongAction::SelfDraw => game.declare_self_draw(core_player),
            AutomaticMahjongAction::ConcealedKong(tile) => {
                game.declare_concealed_kong(core_player, tile)
            }
            AutomaticMahjongAction::AddedKong(tile) => game.declare_added_kong(core_player, tile),
        }
        .ok()?;
        Some((public_event, outcome))
    }

    fn lobby_snapshot(&self) -> LobbySnapshot {
        self.room
            .lobby_snapshot(GameKind::Mahjong, GameRules::Mahjong(self.rules))
    }

    fn broadcast_lobby(&self, origin: Option<(ConnectionId, RequestId)>) -> Vec<Delivery> {
        self.room
            .broadcast_lobby(GameKind::Mahjong, GameRules::Mahjong(self.rules), origin)
    }

    fn broadcast_game(&self, origin: Option<(ConnectionId, RequestId)>) -> Vec<Delivery> {
        assert!(
            self.game.is_some(),
            "game broadcast requires a Mahjong game"
        );
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
                    ServerEvent::GameSnapshot(GameSnapshot::Mahjong(self.game_snapshot(player.id))),
                )
            })
            .collect()
    }

    fn broadcast_events(&self, events: Vec<MahjongEvent>) -> Vec<Delivery> {
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
                            ServerEvent::GameEvent(GameEvent::Mahjong(event.clone())),
                        )
                    })
            })
            .collect()
    }

    fn game_snapshot(&self, recipient: PlayerId) -> MahjongSnapshot {
        let game = self
            .game
            .as_ref()
            .expect("a Mahjong snapshot requires a game");
        let core_recipient = to_core_player(recipient);
        let players = self
            .room
            .players
            .iter()
            .filter(|player| !player.left)
            .map(|participant| {
                let core_id = to_core_player(participant.id);
                let public = game
                    .public_player(core_recipient, core_id)
                    .expect("room and Mahjong players stay aligned");
                MahjongPlayerState {
                    id: participant.id,
                    profile_id: participant.profile_id,
                    name: participant.name.clone(),
                    avatar: participant.avatar,
                    seat: participant.seat.expect("started players retain seats"),
                    seat_wind: game.seat_wind(core_id).expect("started player has a wind"),
                    concealed_count: public.concealed_count as u8,
                    revealed_hand: public.revealed_hand,
                    melds: public
                        .melds
                        .into_iter()
                        .map(|meld| MahjongPublicMeldView {
                            kind: meld.kind,
                            tile: meld.tile,
                            claimed_from: meld.claimed_from.map(from_core_player),
                        })
                        .collect(),
                    flowers: public.flowers,
                    dead_hand: public.dead_hand,
                    ready: participant.ready,
                    connected: participant.connected || participant.is_bot,
                    reference_points: participant.reference_points,
                    completed_games: participant.completed_games,
                    game_profiles: participant.game_profiles.clone(),
                }
            })
            .collect();
        let own = game
            .player(core_recipient)
            .expect("room and Mahjong players stay aligned");
        let pending_claim = match game.phase() {
            Phase::WaitingForClaims(pending) => Some(MahjongPendingClaimView {
                source: from_core_player(pending.source_player()),
                tile: pending.tile(),
                robbing_kong: pending.is_robbing_kong_window(),
                your_options: pending
                    .options_for(core_recipient)
                    .unwrap_or_default()
                    .to_vec(),
                your_response: pending.response_from(core_recipient),
                waiting_for: pending
                    .waiting_for()
                    .into_iter()
                    .map(from_core_player)
                    .collect(),
            }),
            Phase::Dealing { .. }
            | Phase::ReplacingFlower { .. }
            | Phase::Playing
            | Phase::Finished(_) => None,
        };
        let can_declare_kong = matches!(game.phase(), Phase::Playing)
            && game.current_player() == core_recipient
            && game.wall_len() > 0;
        let concealed_kong_options = if can_declare_kong {
            let mut kinds = own
                .hand()
                .iter()
                .map(|tile| tile.kind())
                .collect::<Vec<_>>();
            kinds.sort_unstable();
            kinds
                .iter()
                .copied()
                .collect::<HashSet<_>>()
                .into_iter()
                .filter(|kind| kinds.iter().filter(|held| **held == *kind).count() == 4)
                .collect()
        } else {
            Vec::new()
        };
        let pung_kinds = own
            .melds()
            .iter()
            .filter(|meld| meld.kind() == MeldKind::Pung)
            .map(|meld| meld.tile())
            .collect::<HashSet<_>>();
        let added_kong_options = if can_declare_kong {
            own.hand()
                .iter()
                .copied()
                .filter(|tile| pung_kinds.contains(&tile.kind()))
                .collect()
        } else {
            Vec::new()
        };
        let phase = match game.phase() {
            Phase::Dealing { batch } => MahjongPhaseView::Dealing { batch: *batch },
            Phase::ReplacingFlower { player } => MahjongPhaseView::ReplacingFlower {
                player: from_core_player(*player),
            },
            Phase::Playing => MahjongPhaseView::Playing,
            Phase::WaitingForClaims(_) => MahjongPhaseView::WaitingForClaims,
            Phase::Finished(result) => MahjongPhaseView::Finished {
                result: hand_result_view(result),
            },
        };
        MahjongSnapshot {
            match_id: self.match_id.expect("started Mahjong game has a match id"),
            host_port: self.room.host_port,
            you: recipient,
            host: self
                .room
                .host_player_id()
                .expect("started Mahjong room retains a host"),
            rules: self.rules,
            players,
            your_hand: own.hand().to_vec(),
            your_drawn_tile: (game.current_player() == core_recipient)
                .then(|| game.last_drawn())
                .flatten(),
            discards: game
                .discards()
                .iter()
                .map(|discard| MahjongDiscardView {
                    player: from_core_player(discard.player),
                    tile: discard.tile,
                    claimed_by: discard.claimed_by.map(from_core_player),
                })
                .collect(),
            dealer: from_core_player(game.dealer()),
            prevalent_wind: game.prevalent_wind(),
            sequence_index: game.sequence_index(),
            current_player: from_core_player(game.current_player()),
            wall_len: game.wall_len() as u16,
            match_scores: *game.match_scores(),
            pending_claim,
            can_self_draw: game.self_draw_available(core_recipient).unwrap_or(false),
            concealed_kong_options,
            added_kong_options,
            phase,
        }
    }

    fn reject_game_error(
        &self,
        connection: ConnectionId,
        request_id: RequestId,
        error: &GameError,
    ) -> Vec<Delivery> {
        let violation = match error {
            GameError::InvalidPlayer(_) => MahjongViolation::InvalidPlayer,
            GameError::NotPlayersTurn { .. } => MahjongViolation::NotPlayersTurn,
            GameError::WrongPhase => MahjongViolation::WrongPhase,
            GameError::TileNotInHand(_) => MahjongViolation::TileNotInHand,
            GameError::InvalidClaim => MahjongViolation::InvalidClaim,
            GameError::AlreadyResponded => MahjongViolation::AlreadyResponded,
            GameError::CannotWin => MahjongViolation::CannotWin,
            GameError::CannotKong => MahjongViolation::CannotKong,
            GameError::InvalidRules(_)
            | GameError::InvalidDeckSize { .. }
            | GameError::InvalidDeckContents
            | GameError::Score(_) => {
                return self.room.reject(
                    connection,
                    request_id,
                    RejectReason::InvalidRuleConfiguration,
                );
            }
        };
        self.room.reject(
            connection,
            request_id,
            RejectReason::GameViolation(GameViolation::Mahjong(violation)),
        )
    }
}

fn automatic_mahjong_action(
    game: &GameState,
    player: CorePlayerId,
) -> Option<AutomaticMahjongAction> {
    match game.phase() {
        Phase::WaitingForClaims(pending) => {
            let options = pending.options_for(player)?;
            if options.contains(&ClaimOption::Win) && game.legal_claim_win_available(player).ok()? {
                return Some(AutomaticMahjongAction::Respond(Claim::Win));
            }
            let claim = if options.contains(&ClaimOption::Kong) {
                Claim::Kong
            } else if options.contains(&ClaimOption::Pung) {
                Claim::Pung
            } else if let Some(start) = options.iter().find_map(|option| match option {
                ClaimOption::Chow { start } => Some(*start),
                _ => None,
            }) {
                Claim::Chow { start }
            } else {
                Claim::Pass
            };
            Some(AutomaticMahjongAction::Respond(claim))
        }
        Phase::Playing if game.current_player() == player => {
            if game.legal_self_draw_available(player).ok()? {
                return Some(AutomaticMahjongAction::SelfDraw);
            }
            let state = game.player(player)?;
            if game.wall_len() > 0
                && let Some(kind) = state.hand().iter().map(|tile| tile.kind()).find(|kind| {
                    state
                        .hand()
                        .iter()
                        .filter(|tile| tile.kind() == *kind)
                        .count()
                        == 4
                })
            {
                return Some(AutomaticMahjongAction::ConcealedKong(kind));
            }
            if game.wall_len() > 0
                && let Some(tile) = state.hand().iter().copied().find(|tile| {
                    state
                        .melds()
                        .iter()
                        .any(|meld| meld.kind() == MeldKind::Pung && meld.tile() == tile.kind())
                })
            {
                return Some(AutomaticMahjongAction::AddedKong(tile));
            }
            automatic_discard(state.hand()).map(AutomaticMahjongAction::Discard)
        }
        Phase::Dealing { .. }
        | Phase::ReplacingFlower { .. }
        | Phase::Playing
        | Phase::Finished(_) => None,
    }
}

fn automatic_discard(hand: &[Tile]) -> Option<Tile> {
    hand.iter().copied().min_by_key(|tile| {
        let kind = tile.kind();
        let identical = hand
            .iter()
            .filter(|candidate| candidate.kind() == kind)
            .count()
            .saturating_sub(1) as u16;
        let neighbors = match kind {
            TileKind::Suited { suit, rank } => hand
                .iter()
                .filter_map(|candidate| match candidate.kind() {
                    TileKind::Suited {
                        suit: candidate_suit,
                        rank: candidate_rank,
                    } if candidate_suit == suit => Some(rank.abs_diff(candidate_rank)),
                    _ => None,
                })
                .map(|distance| match distance {
                    1 => 4,
                    2 => 2,
                    _ => 0,
                })
                .sum::<u16>(),
            TileKind::Wind(_) | TileKind::Dragon(_) | TileKind::Flower(_) => 0,
        };
        (identical * 7 + neighbors, kind, tile.copy())
    })
}

fn events_for_outcome(outcome: &ActionOutcome) -> Vec<MahjongEvent> {
    match outcome {
        ActionOutcome::Discarded { .. } | ActionOutcome::ClaimRecorded { .. } => Vec::new(),
        ActionOutcome::Claimed {
            player,
            source,
            tile,
            claim,
        } => vec![MahjongEvent::ClaimResolved {
            player: from_core_player(*player),
            source: from_core_player(*source),
            tile: *tile,
            claim: *claim,
        }],
        ActionOutcome::Drew { player, origin, .. } => vec![MahjongEvent::TileDrawn {
            player: from_core_player(*player),
            origin: *origin,
        }],
        ActionOutcome::KongDeclared {
            player,
            tile,
            added,
        } => vec![MahjongEvent::KongDeclared {
            player: from_core_player(*player),
            tile: *tile,
            added: *added,
        }],
        ActionOutcome::FalseWin { player, deltas } => vec![MahjongEvent::FalseWin {
            player: from_core_player(*player),
            deltas: *deltas,
        }],
        ActionOutcome::HandFinished(result) => vec![MahjongEvent::HandFinished {
            result: hand_result_view(result),
        }],
    }
}

fn hand_result_view(result: &leocard_mahjong::HandResult) -> MahjongHandResultView {
    MahjongHandResultView {
        winners: result
            .winners
            .iter()
            .map(|winner| MahjongWinView {
                player: from_core_player(winner.player),
                from: winner.from.map(from_core_player),
                score: winner.score.clone(),
            })
            .collect(),
        exhaustive_draw: result.exhaustive_draw,
        deltas: result.deltas,
        match_scores: result.match_scores,
        match_complete: result.match_complete,
        sequence_index: result.sequence_index,
    }
}

fn validate_deck(deck: &[Tile]) -> Result<(), HostError> {
    if deck.len() != 144 {
        return Err(HostError::InvalidDeckSize {
            expected: 144,
            actual: deck.len(),
        });
    }
    let mut actual = deck.to_vec();
    actual.sort_unstable();
    let mut expected = build_deck();
    expected.sort_unstable();
    if actual != expected {
        return Err(HostError::InvalidDeckContents);
    }
    Ok(())
}

fn shuffled_deck() -> Vec<Tile> {
    let mut deck = build_deck();
    fastrand::shuffle(&mut deck);
    deck
}

const fn to_core_player(player: PlayerId) -> CorePlayerId {
    CorePlayerId(player.0 as usize)
}

const fn from_core_player(player: CorePlayerId) -> PlayerId {
    PlayerId(player.0 as u8)
}

#[cfg(test)]
mod tests {
    use ed25519_dalek::{Signer, SigningKey};
    use leocard_mahjong::{Suit, TileKind};
    use leocard_protocol::{
        GameSnapshot, MahjongCommand, MahjongPhaseView, ProfileId, ReconnectToken,
        join_identity_payload,
    };

    use super::*;

    const ROOM: RoomId = RoomId(2014);

    fn message(request: u64, command: ClientCommand) -> ClientMessage {
        ClientMessage::new(ROOM, RequestId(request), command)
    }

    fn join_command(name: &str, token: u64) -> ClientCommand {
        let mut secret = [0; 32];
        secret[..8].copy_from_slice(&token.to_be_bytes());
        secret[8] = 14;
        let key = SigningKey::from_bytes(&secret);
        let reconnect_token = ReconnectToken(token);
        let game_profiles = leocard_protocol::PlayerGameProfiles::default();
        let payload = join_identity_payload(ROOM, reconnect_token, name, 0, 0, &game_profiles);
        ClientCommand::Join {
            name: name.to_owned(),
            reconnect_token,
            profile_id: ProfileId(key.verifying_key().to_bytes()),
            reference_points: 0,
            completed_games: 0,
            game_profiles,
            identity_signature: key.sign(&payload).to_bytes().to_vec(),
        }
    }

    fn dealer_winning_deck() -> Vec<Tile> {
        let kinds = [
            (1, 0),
            (1, 1),
            (1, 2),
            (2, 0),
            (2, 1),
            (2, 2),
            (3, 0),
            (3, 1),
            (3, 2),
            (4, 0),
            (4, 1),
            (4, 2),
            (5, 0),
            (5, 1),
        ];
        let selected = kinds
            .into_iter()
            .map(|(rank, copy)| Tile::new(TileKind::suited(Suit::Characters, rank), copy))
            .collect::<Vec<_>>();
        let selected_set = selected.iter().copied().collect::<HashSet<_>>();
        let mut remaining = build_deck()
            .into_iter()
            .filter(|tile| !selected_set.contains(tile))
            .collect::<Vec<_>>();
        let mut deck = Vec::with_capacity(144);
        for round in 0..3 {
            deck.extend_from_slice(&selected[round * 4..round * 4 + 4]);
            for _ in 0..3 {
                deck.extend(remaining.drain(..4));
            }
        }
        deck.push(selected[12]);
        deck.extend(remaining.drain(..3));
        deck.push(selected[13]);
        deck.extend(remaining);
        deck
    }

    fn finish_server_deal(session: &mut MahjongSession) {
        while matches!(
            session.game().expect("game remains active").phase(),
            Phase::Dealing { .. }
        ) {
            assert!(!session.advance_time(MAHJONG_DEAL_INTERVAL).is_empty());
        }
        assert!(matches!(
            session.game().expect("game remains active").phase(),
            Phase::Playing
        ));
    }

    #[test]
    fn four_clients_receive_private_views_and_continue_at_the_table() {
        let mut session = MahjongSession::new(ROOM, RuleSet::default(), dealer_winning_deck())
            .expect("test wall is valid");
        let connections = [
            ConnectionId(10),
            ConnectionId(20),
            ConnectionId(30),
            ConnectionId(40),
        ];
        for (index, connection) in connections.into_iter().enumerate() {
            session.handle(
                connection,
                message(1, join_command(&format!("玩家{index}"), index as u64 + 1)),
            );
        }
        for (index, player) in session.room.players.iter_mut().enumerate() {
            player.seat = Some(leocard_protocol::SeatId(index as u8));
        }
        for connection in connections.into_iter().skip(1) {
            session.handle(
                connection,
                message(2, ClientCommand::SetReady { ready: true }),
            );
        }
        let deliveries = session.handle(connections[0], message(2, ClientCommand::StartGame));
        let snapshots = deliveries
            .iter()
            .filter_map(|delivery| match &delivery.message.event {
                ServerEvent::GameSnapshot(GameSnapshot::Mahjong(snapshot)) => {
                    Some((delivery.recipient, snapshot))
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(snapshots.len(), 4);
        for (_, snapshot) in &snapshots {
            assert_eq!(snapshot.players.len(), 4);
            assert!(matches!(
                snapshot.phase,
                MahjongPhaseView::Dealing { batch: 0 }
            ));
            assert_eq!(snapshot.wall_len, 144);
            assert!(snapshot.your_hand.is_empty());
            assert_eq!(
                snapshot.your_hand.len(),
                usize::from(snapshot.players[snapshot.you.0 as usize].concealed_count)
            );
            assert!(
                snapshot
                    .players
                    .iter()
                    .all(|player| { player.id == snapshot.you || player.revealed_hand.is_none() })
            );
        }

        let dealer_connection = snapshots
            .iter()
            .find_map(|(connection, snapshot)| (snapshot.you == PlayerId(0)).then_some(*connection))
            .expect("one connection owns the dealer seat");
        let first_batch = session.advance_time(MAHJONG_DEAL_INTERVAL);
        let dealer_snapshot = first_batch
            .iter()
            .find_map(|delivery| match &delivery.message.event {
                ServerEvent::GameSnapshot(GameSnapshot::Mahjong(snapshot))
                    if delivery.recipient == dealer_connection =>
                {
                    Some(snapshot)
                }
                _ => None,
            })
            .expect("the first real batch is broadcast to the dealer");
        assert!(matches!(
            dealer_snapshot.phase,
            MahjongPhaseView::Dealing { batch: 1 }
        ));
        assert_eq!(dealer_snapshot.wall_len, 140);
        assert_eq!(dealer_snapshot.your_hand.len(), 4);
        assert_eq!(dealer_snapshot.players[0].concealed_count, 4);
        finish_server_deal(&mut session);
        let finished = session.handle(
            dealer_connection,
            message(
                10,
                ClientCommand::Game(GameCommand::Mahjong(MahjongCommand::DeclareSelfDraw)),
            ),
        );
        assert!(finished.iter().any(|delivery| matches!(
            &delivery.message.event,
            ServerEvent::GameSnapshot(GameSnapshot::Mahjong(MahjongSnapshot {
                phase: MahjongPhaseView::Finished { .. },
                ..
            }))
        )));

        let original_match = session.match_id;
        let mut last = Vec::new();
        for connection in connections {
            last = session.handle(connection, message(20, ClientCommand::PlayAgain));
        }
        assert_ne!(session.match_id, original_match);
        assert!(last.iter().any(|delivery| matches!(
            &delivery.message.event,
            ServerEvent::GameSnapshot(GameSnapshot::Mahjong(MahjongSnapshot {
                phase: MahjongPhaseView::Dealing { batch: 0 },
                ..
            }))
        )));
        assert_eq!(
            session.game().expect("next hand started").sequence_index(),
            1
        );
    }

    #[test]
    fn robot_waits_then_takes_an_automatic_action() {
        let mut session = MahjongSession::new(ROOM, RuleSet::default(), build_deck())
            .expect("standard wall is valid");
        let connections = [
            ConnectionId(110),
            ConnectionId(120),
            ConnectionId(130),
            ConnectionId(140),
        ];
        for (index, connection) in connections.into_iter().enumerate() {
            session.handle(
                connection,
                message(
                    100 + index as u64,
                    join_command(&format!("玩家{index}"), 20 + index as u64),
                ),
            );
        }
        for (index, player) in session.room.players.iter_mut().enumerate() {
            player.seat = Some(leocard_protocol::SeatId(index as u8));
            player.ready = true;
        }
        session.handle(connections[0], message(200, ClientCommand::StartGame));
        session.room.players[0].is_bot = true;
        session.room.players[0].auto_play = true;
        finish_server_deal(&mut session);

        assert!(session.advance_time(Duration::from_millis(999)).is_empty());
        let deliveries = session.advance_time(Duration::from_millis(1));

        assert!(!deliveries.is_empty());
        let game = session.game().expect("game remains active");
        let dealer = game.player(CorePlayerId(0)).unwrap();
        assert!(!game.discards().is_empty() || !dealer.melds().is_empty());
    }
}
