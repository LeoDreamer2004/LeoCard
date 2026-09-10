use super::{
    MAHJONG_DEAL_INTERVAL, MahjongSession, events_for_outcome, shuffled_deck, validate_deck,
};
use crate::lifecycle::{HostedGameLifecycle, dispatch_client_command};
use crate::{
    AUTO_PLAY_DELAY, AutoPlayDelayState, ConnectionId, Delivery, HostError, RoomSession,
    new_match_id,
};
use leocard_mahjong::{GameError, GameState, MahjongPlayerId, MahjongRuleSet, MahjongTile, Phase};
use leocard_protocol::{
    ClientMessage, GameCommand, GameKind, GameRules, GameSnapshot, GameViolation, MahjongEvent,
    MahjongViolation, PlayerId, PlayerInteraction, PlayerInteractionKind, PlayerViolation,
    RejectReason, RequestId, Revision, RoomId, RoomViolation, ServerEvent,
};
use std::time::Duration;

impl MahjongSession {
    pub fn new(
        room_id: RoomId,
        host_port: u16,
        rules: MahjongRuleSet,
        shuffled_deck: Vec<MahjongTile>,
    ) -> Result<Self, HostError> {
        let rules = rules.validate().map_err(GameError::from)?;
        validate_deck(&shuffled_deck)?;
        Ok(Self {
            room: RoomSession::new_with_seat_count(
                room_id,
                host_port,
                MahjongRuleSet::PLAYER_COUNT,
                MahjongRuleSet::PLAYER_COUNT as u8,
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

    pub const fn rules(&self) -> &MahjongRuleSet {
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
            return self.room.broadcast_event(None, ServerEvent::RoomClosed);
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
        dispatch_client_command(self, connection, message)
    }
}

impl HostedGameLifecycle for MahjongSession {
    const KIND: GameKind = GameKind::Mahjong;

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
        MahjongRuleSet::PLAYER_COUNT as u8
    }

    fn game_rules(&self) -> GameRules {
        self.rules.into()
    }

    fn game_snapshot(&self, recipient: PlayerId) -> GameSnapshot {
        MahjongSession::game_snapshot(self, recipient).into()
    }

    fn handle_game_command(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        command: GameCommand,
    ) -> Result<Vec<Delivery>, GameKind> {
        let GameCommand::Mahjong(command) = command else {
            return Err(command.kind());
        };
        Ok(MahjongSession::handle_game_command(
            self, connection, request_id, command,
        ))
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
        if active != MahjongRuleSet::PLAYER_COUNT {
            return self.room.reject(
                connection,
                request_id,
                RejectReason::Room(RoomViolation::WaitingForPlayers {
                    expected: MahjongRuleSet::PLAYER_COUNT as u8,
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
        }
        let deck = self.shuffled_deck.take().unwrap_or_else(shuffled_deck);
        match GameState::new_with_deck(self.rules, deck, MahjongPlayerId(0)) {
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
        if !matches!(game.phase(), Phase::Finished(_)) {
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
        let mut active = self.room.players.iter().filter(|player| !player.left);
        if active.clone().count() == MahjongRuleSet::PLAYER_COUNT
            && active.all(|player| player.ready)
        {
            return self.start_next_hand(Some((connection, request_id)));
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
        if self.game.is_none() {
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
            return self.room.reject(
                connection,
                request_id,
                RejectReason::Game(GameViolation::Mahjong(MahjongViolation::InvalidPlayer)),
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
}
