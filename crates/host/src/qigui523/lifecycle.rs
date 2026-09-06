use super::{QiGui523Session, validate_deck};
use crate::{AUTO_PLAY_DELAY, AutoPlayDelayState, ConnectionId, Delivery, HostError, RoomSession};
use leocard_protocol::{
    ClientCommand, ClientMessage, GameCommand, GameKind, QiGui523Command, RejectReason, Revision,
    RoomId, ServerEvent,
};
use leocard_qigui523::{GameState, QiGuiCard, QiGuiRuleSet};
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
        self.players
            .iter()
            .filter(|player| player.connected)
            .map(|player| self.delivery(player.connection, None, ServerEvent::Heartbeat))
            .collect()
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
            return self
                .players
                .iter()
                .filter(|player| player.connected && !player.left)
                .map(|player| self.delivery(player.connection, None, ServerEvent::RoomClosed))
                .collect();
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
        if let Err(deliveries) = self.room.begin_request(connection, &message) {
            return deliveries;
        }
        // A finished match may be observed again through ready/lobby/snapshot commands.
        // Applying here keeps the settlement idempotent even outside the normal final-play path.
        self.apply_finished_reference_points();

        match message.command {
            ClientCommand::Join(request) => self.join(connection, message.request_id, *request),
            ClientCommand::SetAvatar { png } => {
                self.set_avatar(connection, message.request_id, png)
            }
            ClientCommand::SelectSeat { seat } => {
                self.select_seat(connection, message.request_id, seat)
            }
            ClientCommand::ConfigureBotSeat { seat, occupied } => {
                match self
                    .room
                    .configure_bot_seat(connection, seat, occupied, self.game.is_some())
                {
                    Ok(()) => self.broadcast_lobby(Some((connection, message.request_id))),
                    Err(reason) => self.reject(connection, message.request_id, reason),
                }
            }
            ClientCommand::SetReady { ready } => {
                self.set_ready(connection, message.request_id, ready)
            }
            ClientCommand::Game(GameCommand::QiGui523(command)) => match command {
                QiGui523Command::SetAutoPlay { enabled } => {
                    self.set_auto_play(connection, message.request_id, enabled)
                }
                QiGui523Command::UpdateRules { rules } => {
                    self.update_rules(connection, message.request_id, rules)
                }
                QiGui523Command::PlayCards { cards } => {
                    self.play_cards(connection, message.request_id, &cards)
                }
                QiGui523Command::SetDeveloperHand { cards } => {
                    self.set_developer_hand(connection, message.request_id, cards)
                }
                QiGui523Command::Pass => self.pass(connection, message.request_id),
            },
            ClientCommand::Game(GameCommand::TexasHoldem(_)) => self.reject(
                connection,
                message.request_id,
                RejectReason::WrongGame {
                    expected: GameKind::QiGui523,
                    received: GameKind::TexasHoldem,
                },
            ),
            ClientCommand::Game(GameCommand::Shengji(_)) => self.reject(
                connection,
                message.request_id,
                RejectReason::WrongGame {
                    expected: GameKind::QiGui523,
                    received: GameKind::Shengji,
                },
            ),
            ClientCommand::Game(GameCommand::Uno(_)) => self.reject(
                connection,
                message.request_id,
                RejectReason::WrongGame {
                    expected: GameKind::QiGui523,
                    received: GameKind::Uno,
                },
            ),
            ClientCommand::Game(GameCommand::Mahjong(_)) => self.reject(
                connection,
                message.request_id,
                RejectReason::WrongGame {
                    expected: GameKind::QiGui523,
                    received: GameKind::Mahjong,
                },
            ),
            ClientCommand::StartGame => self.start_game(connection, message.request_id),
            ClientCommand::ReturnToLobby => self.return_to_lobby(connection, message.request_id),
            ClientCommand::PlayAgain => self.play_again(connection, message.request_id),
            ClientCommand::LeaveRoom => self.leave_room(connection, message.request_id),
            ClientCommand::CloseRoom => self.close_room(connection, message.request_id),
            ClientCommand::Interact { target, kind } => {
                self.interact(connection, message.request_id, target, kind)
            }
            ClientCommand::Chat { content } => self.chat(connection, message.request_id, content),
            ClientCommand::RequestSnapshot => self.snapshot(connection, message.request_id),
            ClientCommand::Ping => unreachable!("transport pings are handled by HostSession"),
        }
    }
}
