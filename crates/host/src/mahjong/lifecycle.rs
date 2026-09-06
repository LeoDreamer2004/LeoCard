use super::{MAHJONG_DEAL_INTERVAL, MahjongSession, events_for_outcome, validate_deck};
use crate::{AUTO_PLAY_DELAY, AutoPlayDelayState, ConnectionId, Delivery, HostError, RoomSession};
use leocard_mahjong::{GameError, GameState, MahjongRuleSet, MahjongTile, Phase};
use leocard_protocol::{
    ClientCommand, ClientMessage, GameCommand, GameKind, MahjongEvent, PlayerId, RejectReason,
    Revision, RoomId, ServerEvent,
};
use std::time::Duration;

impl MahjongSession {
    pub fn new(
        room_id: RoomId,
        rules: MahjongRuleSet,
        shuffled_deck: Vec<MahjongTile>,
    ) -> Result<Self, HostError> {
        Self::new_with_host_port(room_id, 52300, rules, shuffled_deck)
    }

    pub fn new_with_host_port(
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
            ClientCommand::Join(request) => match self.room.join(
                connection,
                request_id,
                *request,
                self.game.is_some(),
                MahjongRuleSet::PLAYER_COUNT as u8,
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
}
