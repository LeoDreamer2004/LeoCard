use super::*;
use leocard_protocol::{
    ClientCommand, ClientMessage, GameCommand, GameKind, GameViolation, RejectReason, Revision,
    RoomId, ServerEvent, TABLE_SEAT_COUNT, TexasHoldemCommand,
};
use leocard_texas_holdem::{Phase, TexasHoldemCard, TexasHoldemRuleSet};

impl TexasHoldemSession {
    pub fn new(
        room_id: RoomId,
        rules: TexasHoldemRuleSet,
        shuffled_deck: Vec<TexasHoldemCard>,
    ) -> Result<Self, HostError> {
        Self::new_with_host_port(room_id, 52300, rules, shuffled_deck)
    }

    pub fn new_with_host_port(
        room_id: RoomId,
        host_port: u16,
        rules: TexasHoldemRuleSet,
        shuffled_deck: Vec<TexasHoldemCard>,
    ) -> Result<Self, HostError> {
        let rules = TexasHoldemRuleSet {
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
            ClientCommand::Join(request) => {
                let joined = self.room.join(
                    connection,
                    request_id,
                    *request,
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
                RejectReason::Game(GameViolation::WrongGame {
                    expected: GameKind::TexasHoldem,
                    received: GameKind::QiGui523,
                }),
            ),
            ClientCommand::Game(GameCommand::Shengji(_)) => self.room.reject(
                connection,
                request_id,
                RejectReason::Game(GameViolation::WrongGame {
                    expected: GameKind::TexasHoldem,
                    received: GameKind::Shengji,
                }),
            ),
            ClientCommand::Game(GameCommand::Uno(_)) => self.room.reject(
                connection,
                request_id,
                RejectReason::Game(GameViolation::WrongGame {
                    expected: GameKind::TexasHoldem,
                    received: GameKind::Uno,
                }),
            ),
            ClientCommand::Game(GameCommand::Mahjong(_)) => self.room.reject(
                connection,
                request_id,
                RejectReason::Game(GameViolation::WrongGame {
                    expected: GameKind::TexasHoldem,
                    received: GameKind::Mahjong,
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
