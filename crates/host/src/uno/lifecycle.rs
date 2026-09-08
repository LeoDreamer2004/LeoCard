use super::*;
use leocard_protocol::{
    ClientCommand, ClientMessage, GameCommand, GameKind, GameViolation, RejectReason, Revision,
    RoomId, ServerEvent,
};
use leocard_uno::{GameError, GameState, UnoCard, UnoRuleSet};

impl UnoSession {
    pub fn new(
        room_id: RoomId,
        rules: UnoRuleSet,
        shuffled_deck: Vec<UnoCard>,
    ) -> Result<Self, HostError> {
        Self::new_with_host_port(room_id, 52300, rules, shuffled_deck)
    }

    pub fn new_with_host_port(
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
            ClientCommand::Join(request) => match self.room.join(
                connection,
                request_id,
                *request,
                self.game.is_some(),
                UnoRuleSet::MAX_PLAYERS,
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
                RejectReason::Game(GameViolation::WrongGame {
                    expected: GameKind::Uno,
                    received: command.kind(),
                }),
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
}
