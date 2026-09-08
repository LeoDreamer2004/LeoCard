use super::*;
use leocard_protocol::{
    ClientCommand, ClientMessage, GameCommand, GameKind, GameViolation, RejectReason, RequestId,
    Revision, RoomId, ServerEvent, ShengjiCommand, ShengjiEvent, ShengjiPublicPlay,
    ShengjiThrowFailureStage,
};
use leocard_shengji::BottomCopyState;
use leocard_shengji::{
    ActionOutcome, GameError, GameState, Phase, ShengjiCard, ShengjiRuleSet, TeamProgress,
};

impl ShengjiSession {
    pub fn new(
        room_id: RoomId,
        rules: ShengjiRuleSet,
        shuffled_deck: Vec<ShengjiCard>,
    ) -> Result<Self, HostError> {
        Self::new_with_host_port(room_id, 52300, rules, shuffled_deck)
    }

    pub fn new_with_host_port(
        room_id: RoomId,
        host_port: u16,
        rules: ShengjiRuleSet,
        shuffled_deck: Vec<ShengjiCard>,
    ) -> Result<Self, HostError> {
        let rules = rules.validate().map_err(GameError::from)?;
        validate_deck(&shuffled_deck, rules)?;
        Ok(Self {
            room: RoomSession::new_with_seat_count(
                room_id,
                host_port,
                ShengjiRuleSet::PLAYER_COUNT,
                PLAYER_COUNT,
            ),
            rules,
            shuffled_deck: Some(shuffled_deck),
            game: None,
            match_id: None,
            hand_number: 0,
            teams: TeamProgress::default(),
            next_dealer: None,
            flow: HandFlowState::default(),
            presentation: HeldGamePresentation::default(),
            statistics: HandStatistics::default(),
        })
    }

    pub const fn room_id(&self) -> RoomId {
        self.room.room_id
    }

    pub const fn rules(&self) -> &ShengjiRuleSet {
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
        if self.flow.redeal_remaining.is_some() {
            return self.advance_redeal(elapsed);
        }
        if self.flow.bottom_flip_remaining.is_some() {
            return self.advance_bottom_flip(elapsed);
        }
        if self.presentation.throw_failure.is_some() {
            return self.advance_throw_failure(elapsed);
        }
        if self.presentation.trick.is_some() {
            return self.advance_trick_hold(elapsed);
        }
        let phase = self.game.as_ref().map(|game| game.phase().clone());
        match phase {
            Some(Phase::Dealing) => self.advance_dealing(elapsed),
            Some(Phase::BiddingGrace) => self.advance_bidding(elapsed),
            Some(Phase::BottomFlipping) => self.advance_bottom_flip(elapsed),
            Some(Phase::BottomCopying) => self.advance_bottom_copy(elapsed),
            Some(
                Phase::Burying
                | Phase::BottomCopyBurying
                | Phase::FiveTrumpCrossing
                | Phase::Playing,
            ) => self.advance_automatic_action(elapsed),
            Some(Phase::Finished(_) | Phase::RedealRequired) | None => Vec::new(),
        }
    }

    fn advance_throw_failure(&mut self, elapsed: Duration) -> Vec<Delivery> {
        let held = self
            .presentation
            .throw_failure
            .as_mut()
            .expect("checked above");
        if elapsed < held.remaining {
            held.remaining -= elapsed;
            return Vec::new();
        }
        if held.stage == ShengjiThrowFailureStage::Showing {
            held.stage = ShengjiThrowFailureStage::Returning;
            held.remaining = THROW_FAILURE_RETURN_DURATION;
            self.room.bump_revision();
            return self.broadcast_game(None);
        }

        let held = self
            .presentation
            .throw_failure
            .take()
            .expect("returning throw failure remains present");
        self.reset_automatic_action();
        self.room.bump_revision();
        let mut deliveries = self.broadcast_events(vec![ShengjiEvent::CardsPlayed {
            play: ShengjiPublicPlay {
                player: from_core_player(held.player),
                play: held.forced,
                throw_penalty: held.penalty_points,
            },
            is_lead: true,
        }]);
        deliveries.extend(self.broadcast_game(None));
        deliveries
    }

    fn advance_trick_hold(&mut self, elapsed: Duration) -> Vec<Delivery> {
        let (_, remaining) = self.presentation.trick.as_mut().expect("checked above");
        if elapsed < *remaining {
            *remaining -= elapsed;
            return Vec::new();
        }
        self.presentation.trick = None;
        self.presentation.throw_penalties = [0; ShengjiRuleSet::PLAYER_COUNT];
        self.reset_automatic_action();
        self.room.bump_revision();
        self.broadcast_game(None)
    }

    fn advance_dealing(&mut self, elapsed: Duration) -> Vec<Delivery> {
        self.flow.deal_elapsed += elapsed;
        let mut changed = false;
        let mut events = Vec::new();
        while self.flow.deal_elapsed >= DEAL_INTERVAL {
            self.flow.deal_elapsed -= DEAL_INTERVAL;
            let outcome = self
                .game
                .as_mut()
                .expect("checked above")
                .deal_next()
                .expect("validated double deck deals completely");
            changed = true;
            match outcome {
                ActionOutcome::CardDealt { player, card } => {
                    if let Some(cards) = self.bot_open_cards(player, card)
                        && self
                            .game
                            .as_mut()
                            .expect("game remains active")
                            .declare(player, &cards)
                            .is_ok()
                    {
                        self.record_current_declaration();
                        events.push(ShengjiEvent::DeclarationChanged {
                            declaration: self.declaration_view().unwrap(),
                        });
                    }
                    let dealt = self
                        .game
                        .as_ref()
                        .unwrap()
                        .players()
                        .iter()
                        .map(|player| player.hand.len())
                        .sum::<usize>();
                    if dealt == ShengjiRuleSet::PLAYER_COUNT * self.rules.hand_size() {
                        self.game.as_mut().unwrap().deal_next().unwrap();
                        self.flow.bidding_remaining = Some(BIDDING_GRACE);
                        self.reset_bid_pass_confirmations();
                        events.push(ShengjiEvent::DealCompleted);
                        break;
                    }
                }
                ActionOutcome::DealComplete => {
                    self.flow.bidding_remaining = Some(BIDDING_GRACE);
                    self.reset_bid_pass_confirmations();
                    events.push(ShengjiEvent::DealCompleted);
                    break;
                }
                _ => unreachable!("deal_next only emits deal outcomes"),
            }
        }
        if !changed {
            return Vec::new();
        }
        self.room.bump_revision();
        let mut deliveries = self.broadcast_events(events);
        deliveries.extend(self.broadcast_game(None));
        deliveries
    }

    fn advance_bidding(&mut self, elapsed: Duration) -> Vec<Delivery> {
        self.confirm_automatic_bid_passes();
        if self
            .flow
            .bid_pass_confirmed
            .iter()
            .all(|confirmed| *confirmed)
        {
            return self.finish_bidding(None);
        }
        let remaining = self.flow.bidding_remaining.get_or_insert(BIDDING_GRACE);
        if elapsed < *remaining {
            *remaining -= elapsed;
            self.room.bump_revision();
            return self.broadcast_game(None);
        }
        self.finish_bidding(None)
    }

    pub(super) fn finish_bidding(
        &mut self,
        acknowledgement: Option<(ConnectionId, RequestId)>,
    ) -> Vec<Delivery> {
        self.flow.bidding_remaining = None;
        let locked_declaration = self.declaration_view();
        let outcome = self
            .game
            .as_mut()
            .expect("checked above")
            .close_bidding_and_take_kitty();
        self.room.bump_revision();
        match outcome {
            Ok(ActionOutcome::DealerTookKitty { dealer }) => {
                self.reset_automatic_action();
                let mut deliveries = self.broadcast_events(vec![
                    ShengjiEvent::BiddingLocked {
                        declaration: locked_declaration,
                    },
                    ShengjiEvent::DealerTookKitty {
                        dealer: from_core_player(dealer),
                    },
                ]);
                deliveries.extend(self.broadcast_game(acknowledgement));
                deliveries
            }
            Ok(ActionOutcome::PowerOutageDealerChanged { dealer }) => {
                self.flow.bidding_remaining = Some(POWER_OUTAGE_BIDDING_GRACE);
                self.reset_bid_pass_confirmations();
                let mut events = vec![ShengjiEvent::PowerOutageDealerChanged {
                    dealer: from_core_player(dealer),
                    level: self
                        .game
                        .as_ref()
                        .expect("断电换庄后游戏仍存在")
                        .bidding()
                        .level(),
                }];
                events.extend(self.open_power_outage_bot_declaration());
                let mut deliveries = self.broadcast_events(events);
                deliveries.extend(self.broadcast_game(acknowledgement));
                deliveries
            }
            Ok(ActionOutcome::BottomFlipStarted) => {
                self.flow.bottom_flip_reveal = None;
                self.flow.bottom_flip_remaining = Some(BOTTOM_FLIP_START_DELAY);
                let mut deliveries =
                    self.broadcast_events(vec![ShengjiEvent::BiddingLocked { declaration: None }]);
                deliveries.extend(self.broadcast_game(acknowledgement));
                deliveries
            }
            Err(GameError::RedealRequired) => {
                self.flow.redeal_remaining = Some(REDEAL_DELAY);
                let mut deliveries = self.broadcast_events(vec![ShengjiEvent::RedealRequired]);
                deliveries.extend(self.broadcast_game(acknowledgement));
                deliveries
            }
            Err(_) | Ok(_) => self.broadcast_game(acknowledgement),
        }
    }

    fn advance_bottom_flip(&mut self, elapsed: Duration) -> Vec<Delivery> {
        let remaining = self
            .flow
            .bottom_flip_remaining
            .get_or_insert(BOTTOM_FLIP_START_DELAY);
        if elapsed < *remaining {
            *remaining -= elapsed;
            return Vec::new();
        }
        self.flow.bottom_flip_remaining = None;

        if self
            .flow
            .bottom_flip_reveal
            .as_ref()
            .is_some_and(|reveal| reveal.dealer.is_some())
        {
            let outcome = self
                .game
                .as_mut()
                .expect("扳底展示期间游戏仍然存在")
                .complete_bottom_flip();
            self.flow.bottom_flip_reveal = None;
            if let Ok(ActionOutcome::DealerTookKitty { dealer }) = outcome {
                self.reset_automatic_action();
                self.room.bump_revision();
                let mut deliveries = self.broadcast_events(vec![ShengjiEvent::DealerTookKitty {
                    dealer: from_core_player(dealer),
                }]);
                deliveries.extend(self.broadcast_game(None));
                return deliveries;
            }
        }

        match self.game.as_ref().map(GameState::phase) {
            Some(Phase::Burying) => {
                self.flow.bottom_flip_reveal = None;
                self.reset_automatic_action();
                self.room.bump_revision();
                self.broadcast_game(None)
            }
            Some(Phase::RedealRequired) => {
                self.flow.bottom_flip_reveal = None;
                self.flow.redeal_remaining = Some(REDEAL_DELAY);
                self.room.bump_revision();
                let mut deliveries = self.broadcast_events(vec![ShengjiEvent::RedealRequired]);
                deliveries.extend(self.broadcast_game(None));
                deliveries
            }
            Some(Phase::BottomFlipping) => {
                match self
                    .game
                    .as_mut()
                    .expect("checked above")
                    .flip_next_bottom_card()
                {
                    Ok(ActionOutcome::BottomCardRevealed(reveal)) => {
                        self.flow.bottom_flip_reveal = Some(reveal.clone());
                        self.flow.bottom_flip_remaining = Some(BOTTOM_FLIP_HOLD_DURATION);
                        self.room.bump_revision();
                        let mut deliveries =
                            self.broadcast_events(vec![ShengjiEvent::BottomCardRevealed {
                                reveal: bottom_flip_reveal_view(&reveal),
                            }]);
                        deliveries.extend(self.broadcast_game(None));
                        deliveries
                    }
                    Err(GameError::RedealRequired) => {
                        self.flow.redeal_remaining = Some(REDEAL_DELAY);
                        self.room.bump_revision();
                        let mut deliveries =
                            self.broadcast_events(vec![ShengjiEvent::RedealRequired]);
                        deliveries.extend(self.broadcast_game(None));
                        deliveries
                    }
                    Err(_) | Ok(_) => Vec::new(),
                }
            }
            _ => {
                self.flow.bottom_flip_reveal = None;
                Vec::new()
            }
        }
    }

    fn advance_bottom_copy(&mut self, elapsed: Duration) -> Vec<Delivery> {
        let remaining = self
            .flow
            .bottom_copy_remaining
            .get_or_insert(BOTTOM_COPY_DECISION_TIMEOUT);
        if elapsed < *remaining {
            *remaining -= elapsed;
            let deliveries = self.advance_automatic_action(elapsed);
            if !deliveries.is_empty() {
                return deliveries;
            }
            self.room.bump_revision();
            return self.broadcast_game(None);
        }
        self.flow.bottom_copy_remaining = None;
        let Some(player) = self
            .game
            .as_ref()
            .and_then(GameState::bottom_copy)
            .and_then(BottomCopyState::current)
        else {
            return Vec::new();
        };
        let Ok(outcome) = self
            .game
            .as_mut()
            .expect("checked above")
            .choose_bottom_copy(player, None)
        else {
            return Vec::new();
        };
        self.record_profile_outcome(&outcome);
        self.after_game_outcome(&outcome);
        self.reset_automatic_action();
        self.room.bump_revision();
        let mut deliveries = self.broadcast_events(self.events_for_outcome(&outcome));
        deliveries.extend(self.broadcast_game(None));
        deliveries
    }

    fn advance_redeal(&mut self, elapsed: Duration) -> Vec<Delivery> {
        let remaining = self.flow.redeal_remaining.as_mut().expect("checked above");
        if elapsed < *remaining {
            *remaining -= elapsed;
            return Vec::new();
        }
        self.flow.redeal_remaining = None;
        if self.start_hand(false).is_err() {
            return Vec::new();
        }
        self.room.bump_revision();
        self.broadcast_game(None)
    }

    fn advance_automatic_action(&mut self, elapsed: Duration) -> Vec<Delivery> {
        let Some(player) = self.automatic_player() else {
            self.flow.automatic_action = None;
            return Vec::new();
        };
        let timer = self
            .flow
            .automatic_action
            .get_or_insert((player, AUTOMATIC_ACTION_DELAY));
        if timer.0 != player {
            *timer = (player, AUTOMATIC_ACTION_DELAY);
        }
        if elapsed < timer.1 {
            timer.1 -= elapsed;
            return Vec::new();
        }
        self.flow.automatic_action = None;
        let Some(outcome) = self.play_automatic_action(player) else {
            return Vec::new();
        };
        self.record_profile_outcome(&outcome);
        self.after_game_outcome(&outcome);
        let events = self.events_for_outcome(&outcome);
        self.reset_automatic_action();
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
        self.room.players[index].connected = false;
        if self.game.is_none() {
            self.room.players[index].seat = None;
            self.room.players[index].ready = false;
            self.room.players[index].left = true;
        } else {
            self.reset_automatic_action();
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
            ClientCommand::Join(request) => {
                let joined = self.room.join(
                    connection,
                    request_id,
                    *request,
                    self.game.is_some(),
                    PLAYER_COUNT,
                );
                match joined {
                    Ok(mut deliveries) => {
                        deliveries.extend(if self.game.is_some() {
                            self.broadcast_game(Some((connection, request_id)))
                        } else {
                            self.broadcast_lobby(Some((connection, request_id)))
                        });
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
            ClientCommand::Game(GameCommand::Shengji(command)) => match command {
                ShengjiCommand::SetAutoPlay { enabled } => {
                    self.set_auto_play(connection, request_id, enabled)
                }
                ShengjiCommand::UpdateRules { rules } => {
                    self.update_rules(connection, request_id, rules)
                }
                ShengjiCommand::Declare { cards } => self.declare(connection, request_id, cards),
                ShengjiCommand::ConfirmBidPass => self.confirm_bid_pass(connection, request_id),
                ShengjiCommand::Bury { cards } => self.bury(connection, request_id, cards),
                ShengjiCommand::ChooseBottomCopy { cards } => {
                    self.choose_bottom_copy(connection, request_id, cards)
                }
                ShengjiCommand::ChooseFiveTrumpCrossing { cards } => {
                    self.choose_five_trump_crossing(connection, request_id, cards)
                }
                ShengjiCommand::ReturnFiveTrumpCrossing { cards } => {
                    self.return_five_trump_crossing(connection, request_id, cards)
                }
                ShengjiCommand::PlayCards { cards } => {
                    self.play_cards(connection, request_id, cards)
                }
            },
            ClientCommand::Game(command) => self.room.reject(
                connection,
                request_id,
                RejectReason::Game(GameViolation::WrongGame {
                    expected: GameKind::Shengji,
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
