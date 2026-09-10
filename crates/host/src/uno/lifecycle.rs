use super::{DRAW_REVEAL_INTERVAL, UnoSession, shuffled_uno_deck, to_core_player, validate_deck};
use crate::lifecycle::{HostedGameLifecycle, dispatch_client_command};
use crate::{
    AUTO_PLAY_DELAY, AutoPlayDelayState, ConnectionId, Delivery, HostError, RoomSession,
    new_match_id,
};
use leocard_protocol::{
    ClientMessage, GameCommand, GameKind, GameRules, GameSnapshot, GameViolation, PlayerId,
    PlayerInteraction, PlayerInteractionKind, PlayerViolation, RejectReason, RequestId, Revision,
    RoomId, RoomViolation, ServerEvent, UnoEvent, UnoProfileStats,
};
use leocard_uno::{GameError, GameState, Phase, UnoCard, UnoFlipSide, UnoRuleSet};
use std::time::Duration;

impl UnoSession {
    pub fn new(
        room_id: RoomId,
        host_port: u16,
        rules: UnoRuleSet,
        shuffled_deck: Vec<UnoCard>,
    ) -> Result<Self, HostError> {
        let rules = rules.validate().map_err(GameError::from)?;
        validate_deck(&shuffled_deck, rules)?;
        Ok(Self {
            room: RoomSession::new(room_id, host_port, usize::from(UnoRuleSet::MAX_PLAYERS)),
            rules,
            shuffled_deck: Some(shuffled_deck),
            game: None,
            match_id: None,
            match_profile_stats: Vec::new(),
            finished_reference_changes: None,
            auto_play_delay: None,
            pending_draw_reveal: None,
        })
    }

    pub const fn room_id(&self) -> RoomId {
        self.room.room_id
    }

    pub const fn rules(&self) -> &UnoRuleSet {
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
        self.room.heartbeat()
    }

    pub fn advance_time(&mut self, elapsed: Duration) -> Vec<Delivery> {
        if elapsed.is_zero() || self.game.is_none() {
            return Vec::new();
        }
        if let Some(reveal) = self.pending_draw_reveal.as_mut() {
            if elapsed < reveal.remaining {
                reveal.remaining -= elapsed;
                return Vec::new();
            }
            reveal.revealed = reveal.revealed.saturating_add(1).min(reveal.cards.len());
            reveal.remaining = DRAW_REVEAL_INTERVAL;
            if reveal.revealed == reveal.cards.len() {
                self.pending_draw_reveal = None;
            }
            return self.broadcast_game(None);
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
        let Some((events, draw_reveal)) = self.play_automatic_action() else {
            return Vec::new();
        };
        self.apply_finished_reference_points();
        self.reset_auto_play_delay();
        self.room.bump_revision();
        let mut deliveries = self.broadcast_events(events);
        self.pending_draw_reveal = draw_reveal;
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
            return self.room.broadcast_event(None, ServerEvent::RoomClosed);
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
        dispatch_client_command(self, connection, message)
    }
}

impl HostedGameLifecycle for UnoSession {
    const KIND: GameKind = GameKind::Uno;

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
        UnoRuleSet::MAX_PLAYERS
    }
    fn game_rules(&self) -> GameRules {
        self.rules.into()
    }
    fn game_snapshot(&self, recipient: leocard_protocol::PlayerId) -> GameSnapshot {
        UnoSession::game_snapshot(self, recipient).into()
    }
    fn handle_game_command(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        command: GameCommand,
    ) -> Result<Vec<Delivery>, GameKind> {
        let GameCommand::Uno(command) = command else {
            return Err(command.kind());
        };
        Ok(self.handle_uno_command(connection, request_id, command))
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
        let active = self
            .room
            .players
            .iter()
            .filter(|player| !player.left)
            .count();
        if active < usize::from(UnoRuleSet::MIN_PLAYERS) {
            return self.room.reject(
                connection,
                request_id,
                RejectReason::Room(RoomViolation::NotEnoughPlayers {
                    minimum: UnoRuleSet::MIN_PLAYERS,
                    actual: active as u8,
                }),
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
            .filter(|player| !player.left && !player.ready)
            .map(|player| player.id)
            .collect::<Vec<_>>();
        if !not_ready.is_empty() {
            return self.room.reject(
                connection,
                request_id,
                RejectReason::Room(RoomViolation::PlayersNotReady { players: not_ready }),
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
        let deck = self
            .shuffled_deck
            .take()
            .unwrap_or_else(|| shuffled_uno_deck(self.rules));
        match GameState::new_with_deck(self.rules, active as u8, deck) {
            Ok(game) => {
                let started_on_dark = game.flip_side() == Some(UnoFlipSide::Dark);
                self.game = Some(game);
                self.pending_draw_reveal = None;
                self.match_id = Some(new_match_id());
                self.match_profile_stats = vec![UnoProfileStats::default(); active];
                self.record_state_peaks();
                self.finished_reference_changes = None;
                self.reset_auto_play_delay();
                self.room.bump_revision();
                let mut deliveries = self.broadcast_game(Some((connection, request_id)));
                if started_on_dark {
                    deliveries.extend(self.broadcast_events(vec![UnoEvent::Flipped {
                        side: UnoFlipSide::Dark,
                    }]));
                }
                deliveries
            }
            Err(_) => self.room.reject(
                connection,
                request_id,
                RejectReason::Game(GameViolation::InvalidRuleConfiguration),
            ),
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
        if self.room.host_connection != Some(connection) {
            return self.room.reject(
                connection,
                request_id,
                RejectReason::Room(RoomViolation::OnlyHostCanReturnToLobby),
            );
        }
        if !self
            .game
            .as_ref()
            .is_some_and(|game| matches!(game.phase(), Phase::Finished(_)))
        {
            return self.room.reject(
                connection,
                request_id,
                RejectReason::Game(GameViolation::GameNotFinished),
            );
        }
        self.game = None;
        self.pending_draw_reveal = None;
        self.match_id = None;
        self.match_profile_stats.clear();
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
        self.shuffled_deck = Some(shuffled_uno_deck(self.rules));
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
        if !self
            .game
            .as_ref()
            .is_some_and(|game| matches!(game.phase(), Phase::Finished(_)))
        {
            return self.room.reject(
                connection,
                request_id,
                RejectReason::Game(GameViolation::GameNotFinished),
            );
        }
        if let Some(participant) = self.room.players.iter_mut().find(|item| item.id == player) {
            participant.ready = true;
        }
        self.room.bump_revision();
        let active = self.room.players.iter().filter(|player| !player.left);
        if active.clone().count() >= usize::from(UnoRuleSet::MIN_PLAYERS)
            && active.clone().all(|player| player.ready)
        {
            return self.start_next_game(Some((connection, request_id)));
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
                .broadcast_event(None, ServerEvent::PlayerLeft { name }),
        );
        deliveries.extend(if self.game.is_some() {
            self.broadcast_game(None)
        } else {
            self.broadcast_lobby(None)
        });
        deliveries
    }

    fn interact(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        target: PlayerId,
        kind: PlayerInteractionKind,
    ) -> Vec<Delivery> {
        let Some(source) = self.room.player_id(connection) else {
            return self.room.reject(
                connection,
                request_id,
                RejectReason::Player(PlayerViolation::NotJoined),
            );
        };
        if !self
            .game
            .as_ref()
            .is_some_and(|game| matches!(game.phase(), Phase::Playing))
        {
            return self.room.reject(
                connection,
                request_id,
                RejectReason::Game(GameViolation::GameNotStarted),
            );
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
        self.room.record_received_interaction(target, kind);
        self.room.bump_revision();
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
