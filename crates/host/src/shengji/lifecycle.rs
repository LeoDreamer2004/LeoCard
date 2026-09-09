use super::{
    AUTOMATIC_ACTION_DELAY, BIDDING_GRACE, BOTTOM_COPY_DECISION_TIMEOUT, BOTTOM_FLIP_HOLD_DURATION,
    BOTTOM_FLIP_START_DELAY, DEAL_INTERVAL, HandFlowState, HandStatistics, HeldGamePresentation,
    PLAYER_COUNT, POWER_OUTAGE_BIDDING_GRACE, REDEAL_DELAY, ShengjiSession,
    THROW_FAILURE_RETURN_DURATION, bottom_flip_reveal_view, from_core_player, shuffled_deck,
    validate_deck,
};
use crate::lifecycle::{HostedGameLifecycle, dispatch_client_command};
use crate::{ConnectionId, Delivery, HostError, RoomSession, new_match_id};
use leocard_protocol::{
    ClientMessage, GameCommand, GameKind, GameRules, GameSnapshot, GameViolation, PlayerId,
    PlayerInteraction, PlayerInteractionKind, PlayerViolation, RejectReason, RequestId, Revision,
    RoomId, RoomViolation, ServerEvent, ShengjiCommand, ShengjiEvent, ShengjiProfileStats,
    ShengjiPublicPlay, ShengjiThrowFailureStage, ShengjiViolation,
};
use leocard_shengji::BottomCopyState;
use leocard_shengji::{
    ActionOutcome, GameError, GameState, Phase, ShengjiCard, ShengjiRuleSet, TeamProgress,
};
use std::time::Duration;

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
        self.room.heartbeat()
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
            return self.room.broadcast_event(None, ServerEvent::RoomClosed);
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
        dispatch_client_command(self, connection, message)
    }
}

impl HostedGameLifecycle for ShengjiSession {
    const KIND: GameKind = GameKind::Shengji;

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
        PLAYER_COUNT
    }
    fn game_rules(&self) -> GameRules {
        self.rules.into()
    }
    fn game_snapshot(&self, recipient: leocard_protocol::PlayerId) -> GameSnapshot {
        ShengjiSession::game_snapshot(self, recipient).into()
    }
    fn handle_game_command(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        command: GameCommand,
    ) -> Result<Vec<Delivery>, GameKind> {
        let GameCommand::Shengji(command) = command else {
            return Err(command.kind());
        };
        Ok(match command {
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
            ShengjiCommand::PlayCards { cards } => self.play_cards(connection, request_id, cards),
        })
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
        let active = self.room.players.iter().filter(|player| !player.left);
        let active_count = active.clone().count();
        if active_count < ShengjiRuleSet::PLAYER_COUNT {
            return self.room.reject(
                connection,
                request_id,
                RejectReason::Room(RoomViolation::NotEnoughPlayers {
                    minimum: PLAYER_COUNT,
                    actual: active_count as u8,
                }),
            );
        }
        if active.clone().any(|player| player.seat.is_none()) {
            return self.room.reject(
                connection,
                request_id,
                RejectReason::Room(RoomViolation::MustSelectSeat),
            );
        }
        let not_ready = active
            .filter(|player| !player.ready)
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
        // 核心规则以 0/2、1/3 表示两队，因此开局前必须像座位环一样重排
        // PlayerId。否则玩家按任意顺序选座时，出牌顺序、庄闲归属和结算积分
        // 都可能按加入顺序而不是实际座位计算。
        self.room.players.sort_by_key(|player| {
            player
                .seat
                .expect("all Shengji players selected a seat before starting")
                .0
        });
        for (index, player) in self.room.players.iter_mut().enumerate() {
            player.id = PlayerId(index as u8);
        }
        self.match_id = Some(new_match_id());
        self.hand_number = 0;
        self.teams = TeamProgress::for_rules(&self.rules);
        self.next_dealer = None;
        self.statistics.profiles =
            vec![ShengjiProfileStats::default(); ShengjiRuleSet::PLAYER_COUNT];
        if self.start_hand(true).is_err() {
            #[cfg(feature = "developer")]
            self.room.remove_developer_bots();
            return self.room.reject(
                connection,
                request_id,
                RejectReason::Game(GameViolation::InvalidRuleConfiguration),
            );
        }
        self.room.bump_revision();
        self.broadcast_game(Some((connection, request_id)))
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
        self.statistics.profiles.clear();
        self.flow.automatic_action = None;
        self.flow.bottom_flip_reveal = None;
        self.flow.bottom_flip_remaining = None;
        self.flow.bottom_copy_remaining = None;
        #[cfg(feature = "developer")]
        self.room.remove_developer_bots();
        let host = self.room.host_connection;
        for player in &mut self.room.players {
            player.ready = host == Some(player.connection);
            player.auto_play = player.is_bot;
            if !player.connected {
                player.seat = None;
                player.left = true;
            }
        }
        self.shuffled_deck = Some(shuffled_deck(self.rules));
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
        let active = self
            .room
            .players
            .iter()
            .filter(|player| !player.left && (player.connected || player.is_bot));
        if active.clone().count() == ShengjiRuleSet::PLAYER_COUNT
            && active.clone().all(|player| player.ready)
        {
            for player in &mut self.room.players {
                player.ready = false;
            }
            self.shuffled_deck = Some(shuffled_deck(self.rules));
            if self.start_hand(true).is_err() {
                return self.room.reject(
                    connection,
                    request_id,
                    RejectReason::Game(GameViolation::InvalidRuleConfiguration),
                );
            }
            self.room.bump_revision();
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
        self.reset_automatic_action();
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
                RejectReason::Game(GameViolation::Shengji(ShengjiViolation::InvalidPlayer)),
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
