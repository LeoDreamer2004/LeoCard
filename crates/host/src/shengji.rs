use std::collections::HashSet;
use std::time::Duration;

use leocard_protocol::{
    ClientCommand, ClientMessage, GameCommand, GameEvent, GameKind, GameRules, GameSnapshot,
    GameViolation, LobbySnapshot, MatchId, PlayerId, PlayerInteraction, PlayerInteractionKind,
    PlayerReferenceChange, RejectReason, RequestId, Revision, RoomId, ServerEvent,
    ShengjiBottomFlipMatchView, ShengjiBottomFlipRevealView, ShengjiCommand,
    ShengjiDeclarationView, ShengjiEvent, ShengjiFiveTrumpCrossingStage, ShengjiHandResultView,
    ShengjiPhaseView, ShengjiPlayerState, ShengjiProfileStats, ShengjiPublicPlay, ShengjiSnapshot,
    ShengjiThrowFailureStage, ShengjiThrowFailureView, ShengjiTrickView, ShengjiViolation,
};
#[cfg(test)]
use leocard_shengji::build_deck;
use leocard_shengji::{
    ActionOutcome, BidError, BidKind, BottomFlipReveal, Card, ClassifiedPlay, Component,
    FiveTrumpCrossingStage, FollowError, GameError, GameState, GreedyBot, GreedyBotRequest,
    HandResult, Phase, PlayError, PlayerId as CorePlayerId, RuleSet, TeamProgress, TrickRecord,
    build_deck_for,
};

use crate::{ConnectionId, Delivery, HostError, RoomSession, new_match_id};

const PLAYER_COUNT: u8 = RuleSet::PLAYER_COUNT as u8;
const DEAL_INTERVAL: Duration = Duration::from_millis(100);
const BIDDING_GRACE: Duration = Duration::from_secs(5);
const POWER_OUTAGE_BIDDING_GRACE: Duration = Duration::from_secs(10);
const BOTTOM_FLIP_START_DELAY: Duration = Duration::from_millis(500);
// 扣底不只是一张牌的瞬时提示：客户端还会逐一标出持有同牌的玩家，
// 并在能够判定时强调新庄家。服务端必须保持公开状态足够久，否则下一张底牌
// 会在演出完成前覆盖当前结果。
const BOTTOM_FLIP_HOLD_DURATION: Duration = Duration::from_millis(2800);
const BOTTOM_COPY_DECISION_TIMEOUT: Duration = Duration::from_secs(10);
const AUTOMATIC_ACTION_DELAY: Duration = Duration::from_secs(1);
const REDEAL_DELAY: Duration = Duration::from_millis(650);
const TRICK_HOLD_DURATION: Duration = Duration::from_millis(1200);
const THROW_FAILURE_SHOW_DURATION: Duration = Duration::from_millis(1200);
const THROW_FAILURE_RETURN_DURATION: Duration = Duration::from_millis(420);

#[derive(Clone, Debug)]
struct HeldThrowFailure {
    player: CorePlayerId,
    attempted: Vec<Card>,
    forced: ClassifiedPlay,
    penalty_points: u16,
    stage: ShengjiThrowFailureStage,
    remaining: Duration,
}

/// 四人双升的房主权威会话。发牌、亮主窗口和机器人行动都由房主时钟推进。
#[derive(Clone, Debug)]
pub struct ShengjiSession {
    room: RoomSession,
    rules: RuleSet,
    shuffled_deck: Option<Vec<Card>>,
    game: Option<GameState>,
    match_id: Option<MatchId>,
    hand_number: u32,
    teams: TeamProgress,
    next_dealer: Option<CorePlayerId>,
    deal_elapsed: Duration,
    bidding_remaining: Option<Duration>,
    bid_pass_confirmed: [bool; RuleSet::PLAYER_COUNT],
    bottom_flip_reveal: Option<BottomFlipReveal>,
    bottom_flip_remaining: Option<Duration>,
    bottom_copy_remaining: Option<Duration>,
    automatic_action: Option<(CorePlayerId, Duration)>,
    redeal_remaining: Option<Duration>,
    throw_penalties: [u16; RuleSet::PLAYER_COUNT],
    held_throw_failure: Option<HeldThrowFailure>,
    held_trick: Option<(TrickRecord, Duration)>,
    hand_profile_stats: Vec<ShengjiProfileStats>,
    finished_settlement_id: Option<MatchId>,
    finished_reference_changes: Option<Vec<PlayerReferenceChange>>,
}

impl ShengjiSession {
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
        validate_deck(&shuffled_deck, rules)?;
        Ok(Self {
            room: RoomSession::new_with_seat_count(
                room_id,
                host_port,
                RuleSet::PLAYER_COUNT,
                PLAYER_COUNT,
            ),
            rules,
            shuffled_deck: Some(shuffled_deck),
            game: None,
            match_id: None,
            hand_number: 0,
            teams: TeamProgress::default(),
            next_dealer: None,
            deal_elapsed: Duration::ZERO,
            bidding_remaining: None,
            bid_pass_confirmed: [false; RuleSet::PLAYER_COUNT],
            bottom_flip_reveal: None,
            bottom_flip_remaining: None,
            bottom_copy_remaining: None,
            automatic_action: None,
            redeal_remaining: None,
            throw_penalties: [0; RuleSet::PLAYER_COUNT],
            held_throw_failure: None,
            held_trick: None,
            hand_profile_stats: Vec::new(),
            finished_settlement_id: None,
            finished_reference_changes: None,
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
        if self.redeal_remaining.is_some() {
            return self.advance_redeal(elapsed);
        }
        if self.bottom_flip_remaining.is_some() {
            return self.advance_bottom_flip(elapsed);
        }
        if self.held_throw_failure.is_some() {
            return self.advance_throw_failure(elapsed);
        }
        if self.held_trick.is_some() {
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
        let held = self.held_throw_failure.as_mut().expect("checked above");
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
            .held_throw_failure
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
        let (_, remaining) = self.held_trick.as_mut().expect("checked above");
        if elapsed < *remaining {
            *remaining -= elapsed;
            return Vec::new();
        }
        self.held_trick = None;
        self.throw_penalties = [0; RuleSet::PLAYER_COUNT];
        self.reset_automatic_action();
        self.room.bump_revision();
        self.broadcast_game(None)
    }

    fn advance_dealing(&mut self, elapsed: Duration) -> Vec<Delivery> {
        self.deal_elapsed += elapsed;
        let mut changed = false;
        let mut events = Vec::new();
        while self.deal_elapsed >= DEAL_INTERVAL {
            self.deal_elapsed -= DEAL_INTERVAL;
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
                    if dealt == RuleSet::PLAYER_COUNT * self.rules.hand_size() {
                        self.game.as_mut().unwrap().deal_next().unwrap();
                        self.bidding_remaining = Some(BIDDING_GRACE);
                        self.reset_bid_pass_confirmations();
                        events.push(ShengjiEvent::DealCompleted);
                        break;
                    }
                }
                ActionOutcome::DealComplete => {
                    self.bidding_remaining = Some(BIDDING_GRACE);
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
        if self.bid_pass_confirmed.iter().all(|confirmed| *confirmed) {
            return self.finish_bidding(None);
        }
        let remaining = self.bidding_remaining.get_or_insert(BIDDING_GRACE);
        if elapsed < *remaining {
            *remaining -= elapsed;
            self.room.bump_revision();
            return self.broadcast_game(None);
        }
        self.finish_bidding(None)
    }

    fn finish_bidding(
        &mut self,
        acknowledgement: Option<(ConnectionId, RequestId)>,
    ) -> Vec<Delivery> {
        self.bidding_remaining = None;
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
                self.bidding_remaining = Some(POWER_OUTAGE_BIDDING_GRACE);
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
                self.bottom_flip_reveal = None;
                self.bottom_flip_remaining = Some(BOTTOM_FLIP_START_DELAY);
                let mut deliveries =
                    self.broadcast_events(vec![ShengjiEvent::BiddingLocked { declaration: None }]);
                deliveries.extend(self.broadcast_game(acknowledgement));
                deliveries
            }
            Err(GameError::RedealRequired) => {
                self.redeal_remaining = Some(REDEAL_DELAY);
                let mut deliveries = self.broadcast_events(vec![ShengjiEvent::RedealRequired]);
                deliveries.extend(self.broadcast_game(acknowledgement));
                deliveries
            }
            Err(_) | Ok(_) => self.broadcast_game(acknowledgement),
        }
    }

    fn advance_bottom_flip(&mut self, elapsed: Duration) -> Vec<Delivery> {
        let remaining = self
            .bottom_flip_remaining
            .get_or_insert(BOTTOM_FLIP_START_DELAY);
        if elapsed < *remaining {
            *remaining -= elapsed;
            return Vec::new();
        }
        self.bottom_flip_remaining = None;

        if self
            .bottom_flip_reveal
            .as_ref()
            .is_some_and(|reveal| reveal.dealer.is_some())
        {
            let outcome = self
                .game
                .as_mut()
                .expect("扳底展示期间游戏仍然存在")
                .complete_bottom_flip();
            self.bottom_flip_reveal = None;
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
                self.bottom_flip_reveal = None;
                self.reset_automatic_action();
                self.room.bump_revision();
                self.broadcast_game(None)
            }
            Some(Phase::RedealRequired) => {
                self.bottom_flip_reveal = None;
                self.redeal_remaining = Some(REDEAL_DELAY);
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
                        self.bottom_flip_reveal = Some(reveal.clone());
                        self.bottom_flip_remaining = Some(BOTTOM_FLIP_HOLD_DURATION);
                        self.room.bump_revision();
                        let mut deliveries =
                            self.broadcast_events(vec![ShengjiEvent::BottomCardRevealed {
                                reveal: bottom_flip_reveal_view(&reveal),
                            }]);
                        deliveries.extend(self.broadcast_game(None));
                        deliveries
                    }
                    Err(GameError::RedealRequired) => {
                        self.redeal_remaining = Some(REDEAL_DELAY);
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
                self.bottom_flip_reveal = None;
                Vec::new()
            }
        }
    }

    fn advance_bottom_copy(&mut self, elapsed: Duration) -> Vec<Delivery> {
        let remaining = self
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
        self.bottom_copy_remaining = None;
        let Some(player) = self
            .game
            .as_ref()
            .and_then(GameState::bottom_copy)
            .and_then(leocard_shengji::BottomCopyState::current)
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
        let remaining = self.redeal_remaining.as_mut().expect("checked above");
        if elapsed < *remaining {
            *remaining -= elapsed;
            return Vec::new();
        }
        self.redeal_remaining = None;
        if self.start_hand(false).is_err() {
            return Vec::new();
        }
        self.room.bump_revision();
        self.broadcast_game(None)
    }

    fn advance_automatic_action(&mut self, elapsed: Duration) -> Vec<Delivery> {
        let Some(player) = self.automatic_player() else {
            self.automatic_action = None;
            return Vec::new();
        };
        let timer = self
            .automatic_action
            .get_or_insert((player, AUTOMATIC_ACTION_DELAY));
        if timer.0 != player {
            *timer = (player, AUTOMATIC_ACTION_DELAY);
        }
        if elapsed < timer.1 {
            timer.1 -= elapsed;
            return Vec::new();
        }
        self.automatic_action = None;
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
            ClientCommand::Join {
                name,
                reconnect_token,
                profile_id,
                reference_points,
                completed_games,
                game_profiles,
                identity_signature,
            } => {
                let joined = self.room.join(
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
                RejectReason::WrongGame {
                    expected: GameKind::Shengji,
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
                player.ready = player.is_bot || host == Some(player.connection);
            }
            self.shuffled_deck = Some(shuffled_deck(self.rules));
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
        let active = self.room.players.iter().filter(|player| !player.left);
        let active_count = active.clone().count();
        if active_count < RuleSet::PLAYER_COUNT {
            return self.room.reject(
                connection,
                request_id,
                RejectReason::NotEnoughPlayers {
                    minimum: PLAYER_COUNT,
                    actual: active_count as u8,
                },
            );
        }
        if active.clone().any(|player| player.seat.is_none()) {
            return self
                .room
                .reject(connection, request_id, RejectReason::MustSelectSeat);
        }
        let not_ready = active
            .filter(|player| !player.ready)
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
        self.hand_profile_stats = vec![ShengjiProfileStats::default(); RuleSet::PLAYER_COUNT];
        if self.start_hand(true).is_err() {
            #[cfg(feature = "developer")]
            self.room.remove_developer_bots();
            return self.room.reject(
                connection,
                request_id,
                RejectReason::InvalidRuleConfiguration,
            );
        }
        self.room.bump_revision();
        self.broadcast_game(Some((connection, request_id)))
    }

    fn start_hand(&mut self, count_as_next_hand: bool) -> Result<(), GameError> {
        let deck = self
            .shuffled_deck
            .take()
            .unwrap_or_else(|| shuffled_deck(self.rules));
        let fixed_dealer = self.next_dealer;
        self.game = Some(GameState::new(
            self.rules,
            self.teams.clone(),
            fixed_dealer,
            fixed_dealer.unwrap_or(CorePlayerId(0)),
            deck,
        )?);
        if count_as_next_hand || self.hand_number == 0 {
            self.hand_number = self.hand_number.saturating_add(1);
        } else {
            // 无人亮主重新发牌也需要让客户端重置逐张发牌动画。
            self.hand_number = self.hand_number.saturating_add(1);
        }
        self.deal_elapsed = Duration::ZERO;
        self.bidding_remaining = None;
        self.bid_pass_confirmed = [false; RuleSet::PLAYER_COUNT];
        self.bottom_flip_reveal = None;
        self.bottom_flip_remaining = None;
        self.bottom_copy_remaining = None;
        self.automatic_action = None;
        self.redeal_remaining = None;
        self.throw_penalties = [0; RuleSet::PLAYER_COUNT];
        self.held_throw_failure = None;
        self.held_trick = None;
        self.hand_profile_stats.fill(ShengjiProfileStats::default());
        self.finished_settlement_id = None;
        self.finished_reference_changes = None;
        Ok(())
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
        if self.game.is_none() {
            return self
                .room
                .reject(connection, request_id, RejectReason::GameNotStarted);
        }
        let participant = self
            .room
            .players
            .iter_mut()
            .find(|participant| participant.id == player)
            .expect("joined player remains in the room");
        if participant.auto_play != enabled {
            participant.auto_play = enabled;
            self.room.bump_revision();
        }
        self.reset_automatic_action();
        self.broadcast_game(Some((connection, request_id)))
    }

    fn declare(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        cards: Vec<Card>,
    ) -> Vec<Delivery> {
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
        match game.declare(to_core_player(player), &cards) {
            Ok(_) => {
                self.record_current_declaration();
                self.reset_bid_pass_confirmations();
                self.room.bump_revision();
                let event = ShengjiEvent::DeclarationChanged {
                    declaration: self.declaration_view().unwrap(),
                };
                let mut deliveries = self.broadcast_events(vec![event]);
                deliveries.extend(self.broadcast_game(Some((connection, request_id))));
                deliveries
            }
            Err(error) => self.reject_game_error(connection, request_id, error),
        }
    }

    fn confirm_bid_pass(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
    ) -> Vec<Delivery> {
        let Some(player) = self.room.player_id(connection) else {
            return self
                .room
                .reject(connection, request_id, RejectReason::NotJoined);
        };
        if !self
            .game
            .as_ref()
            .is_some_and(|game| matches!(game.phase(), Phase::BiddingGrace))
        {
            return self.reject_game_error(connection, request_id, GameError::WrongPhase);
        }
        self.bid_pass_confirmed[usize::from(player.0)] = true;
        self.confirm_automatic_bid_passes();
        self.room.bump_revision();
        if self.bid_pass_confirmed.iter().all(|confirmed| *confirmed) {
            self.finish_bidding(Some((connection, request_id)))
        } else {
            self.broadcast_game(Some((connection, request_id)))
        }
    }

    fn reset_bid_pass_confirmations(&mut self) {
        self.bid_pass_confirmed = [false; RuleSet::PLAYER_COUNT];
        self.confirm_automatic_bid_passes();
    }

    fn confirm_automatic_bid_passes(&mut self) {
        for participant in &self.room.players {
            if usize::from(participant.id.0) >= RuleSet::PLAYER_COUNT {
                continue;
            }
            if participant.is_bot
                || participant.auto_play
                || !participant.connected
                || participant.left
            {
                self.bid_pass_confirmed[usize::from(participant.id.0)] = true;
            }
        }
    }

    fn bury(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        cards: Vec<Card>,
    ) -> Vec<Delivery> {
        self.apply_card_action(connection, request_id, |game, player| {
            game.bury(player, &cards)
        })
    }

    fn choose_bottom_copy(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        cards: Option<Vec<Card>>,
    ) -> Vec<Delivery> {
        self.apply_card_action(connection, request_id, |game, player| {
            game.choose_bottom_copy(player, cards.as_deref())
        })
    }

    fn choose_five_trump_crossing(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        cards: Option<Vec<Card>>,
    ) -> Vec<Delivery> {
        self.apply_card_action(connection, request_id, |game, player| {
            game.choose_five_trump_crossing(player, cards.as_deref())
        })
    }

    fn return_five_trump_crossing(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        cards: Vec<Card>,
    ) -> Vec<Delivery> {
        self.apply_card_action(connection, request_id, |game, player| {
            game.return_five_trump_crossing(player, &cards)
        })
    }

    fn play_cards(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        cards: Vec<Card>,
    ) -> Vec<Delivery> {
        self.apply_card_action(connection, request_id, |game, player| {
            game.play_cards(player, &cards)
        })
    }

    fn apply_card_action(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        action: impl FnOnce(&mut GameState, CorePlayerId) -> Result<ActionOutcome, GameError>,
    ) -> Vec<Delivery> {
        if self.held_trick.is_some() || self.held_throw_failure.is_some() {
            return self.room.reject(
                connection,
                request_id,
                RejectReason::GameViolation(GameViolation::Shengji(ShengjiViolation::WrongPhase)),
            );
        }
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
        let outcome = match action(game, to_core_player(player)) {
            Ok(outcome) => outcome,
            Err(error) => return self.reject_game_error(connection, request_id, error),
        };
        self.record_profile_outcome(&outcome);
        self.after_game_outcome(&outcome);
        let events = self.events_for_outcome(&outcome);
        self.reset_automatic_action();
        self.room.bump_revision();
        let mut deliveries = self.broadcast_events(events);
        deliveries.extend(self.broadcast_game(Some((connection, request_id))));
        deliveries
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
        self.hand_profile_stats.clear();
        self.automatic_action = None;
        self.bottom_flip_reveal = None;
        self.bottom_flip_remaining = None;
        self.bottom_copy_remaining = None;
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
        let active = self
            .room
            .players
            .iter()
            .filter(|player| !player.left && (player.connected || player.is_bot));
        if active.clone().count() == RuleSet::PLAYER_COUNT
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
                    RejectReason::InvalidRuleConfiguration,
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
        self.reset_automatic_action();
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
                RejectReason::GameViolation(GameViolation::Shengji(
                    ShengjiViolation::InvalidPlayer,
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
            ServerEvent::GameSnapshot(GameSnapshot::Shengji(self.game_snapshot(player)))
        } else {
            ServerEvent::LobbySnapshot(self.lobby_snapshot())
        };
        vec![self.room.delivery(connection, Some(request_id), event)]
    }

    fn lobby_snapshot(&self) -> LobbySnapshot {
        self.room
            .lobby_snapshot(GameKind::Shengji, GameRules::Shengji(self.rules))
    }

    fn broadcast_lobby(&self, origin: Option<(ConnectionId, RequestId)>) -> Vec<Delivery> {
        self.room
            .broadcast_lobby(GameKind::Shengji, GameRules::Shengji(self.rules), origin)
    }

    fn broadcast_game(&self, origin: Option<(ConnectionId, RequestId)>) -> Vec<Delivery> {
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
                    ServerEvent::GameSnapshot(GameSnapshot::Shengji(self.game_snapshot(player.id))),
                )
            })
            .collect()
    }

    fn game_snapshot(&self, recipient: PlayerId) -> ShengjiSnapshot {
        let game = self.game.as_ref().expect("game snapshot requires a game");
        let core_recipient = to_core_player(recipient);
        let your_hand = game.players()[usize::from(core_recipient.0)].hand.clone();
        let visible_trick = game
            .current_trick()
            .or_else(|| self.held_trick.as_ref().map(|(trick, _)| trick.clone()));
        let trick = visible_trick.map(|trick| ShengjiTrickView {
            leader: from_core_player(trick.leader),
            current_player: from_core_player(game.current_player().unwrap_or(trick.leader)),
            winning_player: from_core_player(trick.winner),
            plays: trick
                .plays
                .into_iter()
                .map(|(player, play)| ShengjiPublicPlay {
                    player: from_core_player(player),
                    play,
                    throw_penalty: self.throw_penalties[usize::from(player.0)],
                })
                .collect(),
            table_points: trick.points,
        });
        ShengjiSnapshot {
            match_id: self.match_id.expect("started game has a match id"),
            hand_number: self.hand_number,
            host_port: self.room.host_port,
            you: recipient,
            host: self.room.host_player_id().expect("started room has a host"),
            rules: self.rules,
            players: self
                .room
                .players
                .iter()
                .filter(|player| {
                    !player.left || game.players().get(usize::from(player.id.0)).is_some()
                })
                .map(|player| ShengjiPlayerState {
                    id: player.id,
                    profile_id: player.profile_id,
                    name: player.name.clone(),
                    avatar: player.avatar,
                    seat: player.seat.expect("started player has a seat"),
                    hand_len: game
                        .players()
                        .get(usize::from(player.id.0))
                        .map_or(0, |state| state.hand.len() as u8),
                    ready: player.ready,
                    connected: (player.connected || player.is_bot) && !player.left,
                    auto_play: player.auto_play,
                    reference_points: player.reference_points,
                    completed_games: player.completed_games,
                    game_profiles: player.game_profiles.clone(),
                })
                .collect(),
            your_hand,
            your_exposed_cards: game.bidding().exposed_cards(core_recipient),
            levels: game.teams().levels,
            bidding_level: game.bidding().level(),
            dealer: game.dealer().map(from_core_player),
            trump: game.trump(),
            declaration: self.declaration_view(),
            current_player: (self.held_trick.is_none()
                && self.held_throw_failure.is_none()
                && self.bottom_flip_remaining.is_none())
            .then(|| game.current_player().map(from_core_player))
            .flatten(),
            trick,
            throw_failure: self.held_throw_failure.as_ref().map(|failure| {
                ShengjiThrowFailureView {
                    player: from_core_player(failure.player),
                    attempted: failure.attempted.clone(),
                    forced: failure.forced.clone(),
                    penalty_points: failure.penalty_points,
                    stage: failure.stage,
                }
            }),
            collecting_score: if self.held_trick.is_some() {
                match game.phase() {
                    Phase::Finished(result) => pre_kitty_collecting_score(result),
                    _ => game.collecting_score(),
                }
            } else {
                game.collecting_score()
            },
            buried_count: game.buried().len() as u8,
            your_buried: (game.bottom_burier() == Some(core_recipient))
                .then(|| game.buried().to_vec())
                .unwrap_or_default(),
            phase: self.phase_view(game, recipient),
        }
    }

    fn phase_view(&self, game: &GameState, recipient: PlayerId) -> ShengjiPhaseView {
        if self.bottom_flip_remaining.is_some() {
            return self.bottom_flip_phase_view();
        }
        if self.held_trick.is_some() && matches!(game.phase(), Phase::Finished(_)) {
            return ShengjiPhaseView::Playing;
        }
        match game.phase() {
            Phase::Dealing => {
                let dealt = game
                    .players()
                    .iter()
                    .map(|player| player.hand.len())
                    .sum::<usize>();
                ShengjiPhaseView::Dealing {
                    cards_remaining: (RuleSet::PLAYER_COUNT * self.rules.hand_size() - dealt) as u8,
                }
            }
            Phase::BiddingGrace => ShengjiPhaseView::BiddingGrace {
                milliseconds_remaining: self
                    .bidding_remaining
                    .unwrap_or(BIDDING_GRACE)
                    .as_millis()
                    .min(u128::from(u16::MAX)) as u16,
                power_outage: game.power_outage_used(),
                confirmed_count: self
                    .bid_pass_confirmed
                    .iter()
                    .filter(|confirmed| **confirmed)
                    .count() as u8,
                you_confirmed: self
                    .bid_pass_confirmed
                    .get(usize::from(recipient.0))
                    .copied()
                    .unwrap_or(false),
            },
            Phase::BottomFlipping => self.bottom_flip_phase_view(),
            Phase::Burying => ShengjiPhaseView::Burying,
            Phase::BottomCopying => ShengjiPhaseView::BottomCopying {
                player: from_core_player(
                    game.bottom_copy()
                        .and_then(leocard_shengji::BottomCopyState::current)
                        .expect("抄底询问阶段始终有当前玩家"),
                ),
                milliseconds_remaining: self
                    .bottom_copy_remaining
                    .unwrap_or(BOTTOM_COPY_DECISION_TIMEOUT)
                    .as_millis()
                    .min(u128::from(u16::MAX)) as u16,
            },
            Phase::BottomCopyBurying => ShengjiPhaseView::BottomCopyBurying {
                player: from_core_player(
                    game.bottom_copy()
                        .and_then(leocard_shengji::BottomCopyState::bottom_holder)
                        .expect("抄底重埋阶段始终有持底玩家"),
                ),
            },
            Phase::FiveTrumpCrossing => {
                let crossing = game
                    .five_trump_crossing()
                    .expect("过江阶段始终保留过江状态");
                let players = (0..RuleSet::PLAYER_COUNT as u8).map(CorePlayerId);
                let eligible = players
                    .clone()
                    .filter(|player| crossing.eligible(*player))
                    .map(from_core_player)
                    .collect();
                let decided = players
                    .clone()
                    .filter(|player| crossing.decision_made(*player))
                    .map(from_core_player)
                    .collect();
                // 决定阶段只公布谁已经完成选择，不提前泄露其是否过江。
                let crossing_players = if crossing.stage() == FiveTrumpCrossingStage::Returning {
                    players
                        .clone()
                        .filter(|player| crossing.crossing(*player))
                        .map(from_core_player)
                        .collect()
                } else {
                    Vec::new()
                };
                let returned = players
                    .filter(|player| crossing.return_made(*player))
                    .map(from_core_player)
                    .collect();
                ShengjiPhaseView::FiveTrumpCrossing {
                    stage: match crossing.stage() {
                        FiveTrumpCrossingStage::Deciding => ShengjiFiveTrumpCrossingStage::Deciding,
                        FiveTrumpCrossingStage::Returning => {
                            ShengjiFiveTrumpCrossingStage::Returning
                        }
                    },
                    eligible,
                    decided,
                    crossing: crossing_players,
                    returned,
                }
            }
            Phase::Playing => ShengjiPhaseView::Playing,
            Phase::Finished(result) => ShengjiPhaseView::Finished {
                result: self.hand_result_view(result),
                buried: game.buried().to_vec(),
            },
            Phase::RedealRequired => ShengjiPhaseView::Redealing,
        }
    }

    fn bottom_flip_phase_view(&self) -> ShengjiPhaseView {
        ShengjiPhaseView::BottomFlipping {
            reveal: self
                .bottom_flip_reveal
                .as_ref()
                .map(bottom_flip_reveal_view),
        }
    }

    fn declaration_view(&self) -> Option<ShengjiDeclarationView> {
        let declaration = self.game.as_ref()?.bidding().current()?;
        Some(ShengjiDeclarationView {
            player: from_core_player(declaration.player),
            trump: declaration.trump,
            kind: declaration.kind,
            protected: declaration.protected,
            cards: declaration.cards.clone(),
        })
    }

    fn bot_open_cards(&self, player: CorePlayerId, dealt_card: Card) -> Option<Vec<Card>> {
        let participant = self.room.players.get(usize::from(player.0))?;
        let game = self.game.as_ref()?;
        if !participant.is_bot || game.bidding().current().is_some() {
            return None;
        }
        let level = game.bidding().level();
        if !game.rules().bid_with_joker {
            return (dealt_card.rank() == level).then(|| vec![dealt_card]);
        }
        let hand = &game.players().get(usize::from(player.0))?.hand;
        hand.iter().copied().find_map(|level_card| {
            let suit = (level_card.rank() == level)
                .then(|| level_card.suit())
                .flatten()?;
            let joker_rank = leocard_shengji::bid_joker_for_suit(suit);
            let joker = hand
                .iter()
                .copied()
                .find(|card| card.rank() == joker_rank && card.suit().is_none())?;
            Some(vec![level_card, joker])
        })
    }

    fn open_power_outage_bot_declaration(&mut self) -> Option<ShengjiEvent> {
        let bot_players = self
            .room
            .players
            .iter()
            .filter(|participant| participant.is_bot && !participant.left)
            .map(|participant| to_core_player(participant.id))
            .collect::<Vec<_>>();
        for player in bot_players {
            let cards = {
                let game = self.game.as_ref()?;
                if game.bidding().current().is_some() {
                    return None;
                }
                let level = game.bidding().level();
                let hand = &game.players().get(usize::from(player.0))?.hand;
                if !game.rules().bid_with_joker {
                    hand.iter()
                        .copied()
                        .find(|card| card.rank() == level)
                        .map(|card| vec![card])
                } else {
                    hand.iter().copied().find_map(|level_card| {
                        let suit = (level_card.rank() == level)
                            .then(|| level_card.suit())
                            .flatten()?;
                        let joker_rank = leocard_shengji::bid_joker_for_suit(suit);
                        let joker = hand
                            .iter()
                            .copied()
                            .find(|card| card.rank() == joker_rank && card.suit().is_none())?;
                        Some(vec![level_card, joker])
                    })
                }
            };
            if let Some(cards) = cards
                && self.game.as_mut()?.declare(player, &cards).is_ok()
            {
                self.record_current_declaration();
                return Some(ShengjiEvent::DeclarationChanged {
                    declaration: self.declaration_view()?,
                });
            }
        }
        None
    }

    fn automatic_player(&self) -> Option<CorePlayerId> {
        if self.held_trick.is_some() || self.held_throw_failure.is_some() {
            return None;
        }
        let game = self.game.as_ref()?;
        let player = match game.phase() {
            Phase::Burying => game.dealer()?,
            Phase::BottomCopying => game.bottom_copy()?.current()?,
            Phase::BottomCopyBurying => game.bottom_copy()?.bottom_holder()?,
            Phase::FiveTrumpCrossing => game
                .five_trump_crossing()?
                .pending_players()
                .into_iter()
                .find(|player| self.is_automatic_participant(*player))?,
            Phase::Playing => game.current_player()?,
            _ => return None,
        };
        self.is_automatic_participant(player).then_some(player)
    }

    fn is_automatic_participant(&self, player: CorePlayerId) -> bool {
        self.room
            .players
            .get(usize::from(player.0))
            .is_some_and(|participant| {
                participant.is_bot
                    || participant.auto_play
                    || !participant.connected
                    || participant.left
            })
    }

    fn reset_automatic_action(&mut self) {
        self.automatic_action = self
            .automatic_player()
            .map(|player| (player, AUTOMATIC_ACTION_DELAY));
    }

    fn play_automatic_action(&mut self, player: CorePlayerId) -> Option<ActionOutcome> {
        let game = self.game.as_mut()?;
        let outcome = match game.phase() {
            Phase::Burying => {
                let trump = game.trump()?;
                let mut hand = game.players()[usize::from(player.0)].hand.clone();
                hand.sort_by_key(|card| card_sort_key(*card, trump));
                game.bury(player, &hand[..game.rules().kitty_size()]).ok()?
            }
            Phase::BottomCopying => game.choose_bottom_copy(player, None).ok()?,
            Phase::BottomCopyBurying => {
                let trump = game.trump()?;
                let mut hand = game.players()[usize::from(player.0)].hand.clone();
                hand.sort_by_key(|card| card_sort_key(*card, trump));
                game.bury(player, &hand[..game.rules().kitty_size()]).ok()?
            }
            Phase::FiveTrumpCrossing => match game.five_trump_crossing()?.stage() {
                FiveTrumpCrossingStage::Deciding => {
                    game.choose_five_trump_crossing(player, None).ok()?
                }
                FiveTrumpCrossingStage::Returning => {
                    let mut hand = game.players()[usize::from(player.0)].hand.clone();
                    let trump = game.trump()?;
                    hand.sort_by_key(|card| card_sort_key(*card, trump));
                    game.return_five_trump_crossing(player, &hand[..5]).ok()?
                }
            },
            Phase::Playing => {
                let trick = game.current_trick();
                let play = GreedyBot::choose(GreedyBotRequest {
                    hand: &game.players()[usize::from(player.0)].hand,
                    lead: trick.as_ref().map(|trick| &trick.plays[0].1),
                    trump: game.trump()?,
                })
                .ok()?;
                game.play_cards(player, &play.cards).ok()?
            }
            _ => return None,
        };
        Some(outcome)
    }

    fn record_current_declaration(&mut self) {
        let Some((player, kind)) = self
            .game
            .as_ref()
            .and_then(|game| game.bidding().current())
            .map(|declaration| (declaration.player, declaration.kind))
        else {
            return;
        };
        let Some(stats) = self.hand_profile_stats.get_mut(usize::from(player.0)) else {
            return;
        };
        match kind {
            BidKind::Initial => stats.declaration_games = 1,
            BidKind::Counter | BidKind::SelfCounter => stats.counter_games = 1,
            BidKind::Protect => {}
        }
    }

    fn record_profile_outcome(&mut self, outcome: &ActionOutcome) {
        match outcome {
            ActionOutcome::BottomCopyDecision {
                player,
                copied: true,
                ..
            } => {
                if let Some(stats) = self.hand_profile_stats.get_mut(usize::from(player.0)) {
                    stats.counter_games = 1;
                }
            }
            ActionOutcome::FiveTrumpCrossingDecision {
                player,
                crossing: true,
                ..
            } => {
                if let Some(stats) = self.hand_profile_stats.get_mut(usize::from(player.0)) {
                    stats.crossing_games = 1;
                }
            }
            ActionOutcome::Played { player, .. } | ActionOutcome::ThrowFailed { player, .. } => {
                let play =
                    self.game
                        .as_ref()
                        .and_then(GameState::current_trick)
                        .and_then(|trick| {
                            trick.plays.last().map(|(_, play)| {
                                (play.clone(), trick.leader == *player, trick.winner)
                            })
                        });
                if let Some((play, is_lead, winner)) = play {
                    self.record_profile_play(*player, &play, is_lead, winner == *player);
                }
            }
            ActionOutcome::TrickComplete(trick) => {
                if let Some((player, play)) = trick.plays.last() {
                    self.record_profile_play(
                        *player,
                        play,
                        trick.leader == *player,
                        trick.winner == *player,
                    );
                }
            }
            ActionOutcome::HandComplete(_) => {
                let play = self
                    .game
                    .as_ref()
                    .and_then(|game| game.history().last())
                    .and_then(|trick| {
                        trick.plays.last().map(|(player, play)| {
                            (*player, play.clone(), trick.leader, trick.winner)
                        })
                    });
                if let Some((player, play, leader, winner)) = play {
                    self.record_profile_play(player, &play, leader == player, winner == player);
                }
            }
            _ => {}
        }
    }

    fn record_profile_play(
        &mut self,
        player: CorePlayerId,
        play: &ClassifiedPlay,
        is_lead: bool,
        is_winning: bool,
    ) {
        let Some(stats) = self.hand_profile_stats.get_mut(usize::from(player.0)) else {
            return;
        };
        stats.plays = stats.plays.saturating_add(1);
        if is_winning {
            stats.winning_plays = stats.winning_plays.saturating_add(1);
        }
        if is_lead && play.is_throw() {
            stats.play_category_counts[4] = stats.play_category_counts[4].saturating_add(1);
            stats.longest_throw = stats
                .longest_throw
                .max(play.cards.len().min(usize::from(u16::MAX)) as u16);
            for component in &play.components {
                record_shengji_component(stats, component);
            }
        } else {
            record_shengji_component(stats, play.strongest_component());
        }
    }

    fn events_for_outcome(&self, outcome: &ActionOutcome) -> Vec<ShengjiEvent> {
        match outcome {
            ActionOutcome::BuryComplete { leader } => vec![ShengjiEvent::CardsBuried {
                dealer: from_core_player(*leader),
            }],
            ActionOutcome::BottomCopyDecision { copied: true, .. } => self
                .declaration_view()
                .map(|declaration| vec![ShengjiEvent::BottomCopied { declaration }])
                .unwrap_or_default(),
            ActionOutcome::BottomCopyBuryComplete { player, .. } => {
                vec![ShengjiEvent::CardsBuried {
                    dealer: from_core_player(*player),
                }]
            }
            ActionOutcome::FiveTrumpCrossingDecision {
                decisions_complete: true,
                ..
            } => self
                .game
                .as_ref()
                .and_then(GameState::five_trump_crossing)
                .filter(|state| state.stage() == FiveTrumpCrossingStage::Returning)
                .map(|state| ShengjiEvent::FiveTrumpCrossingStarted {
                    players: (0..RuleSet::PLAYER_COUNT as u8)
                        .map(CorePlayerId)
                        .filter(|player| state.crossing(*player))
                        .map(from_core_player)
                        .collect(),
                })
                .into_iter()
                .collect(),
            ActionOutcome::FiveTrumpCrossingReturn {
                player,
                crossing_complete,
            } => vec![ShengjiEvent::FiveTrumpCrossingReturned {
                player: from_core_player(*player),
                complete: *crossing_complete,
            }],
            ActionOutcome::Played { player, .. } => self
                .current_public_play(*player, 0)
                .map(|(play, is_lead)| vec![ShengjiEvent::CardsPlayed { play, is_lead }])
                .unwrap_or_default(),
            ActionOutcome::ThrowFailed { .. } => Vec::new(),
            ActionOutcome::TrickComplete(trick) => {
                completed_trick_events(trick, self.game.as_ref().unwrap().collecting_score())
            }
            ActionOutcome::HandComplete(result) => {
                let game = self.game.as_ref().unwrap();
                let mut events = game
                    .history()
                    .last()
                    .map(|trick| completed_trick_events(trick, pre_kitty_collecting_score(result)))
                    .unwrap_or_default();
                events.push(ShengjiEvent::HandFinished {
                    result: self.hand_result_view(result),
                    buried: game.buried().to_vec(),
                });
                events
            }
            _ => Vec::new(),
        }
    }

    fn current_public_play(
        &self,
        player: CorePlayerId,
        throw_penalty: u16,
    ) -> Option<(ShengjiPublicPlay, bool)> {
        let trick = self.game.as_ref()?.current_trick()?;
        let play = trick.plays.last()?.1.clone();
        let is_lead = trick.plays.len() == 1;
        Some((
            ShengjiPublicPlay {
                player: from_core_player(player),
                play,
                throw_penalty,
            },
            is_lead,
        ))
    }

    fn after_game_outcome(&mut self, outcome: &ActionOutcome) {
        match outcome {
            ActionOutcome::BuryComplete { .. } => {
                self.bottom_copy_remaining = self
                    .game
                    .as_ref()
                    .is_some_and(|game| matches!(game.phase(), Phase::BottomCopying))
                    .then_some(BOTTOM_COPY_DECISION_TIMEOUT);
            }
            ActionOutcome::BottomCopyDecision { copied, .. } => {
                self.bottom_copy_remaining = (!*copied
                    && self
                        .game
                        .as_ref()
                        .is_some_and(|game| matches!(game.phase(), Phase::BottomCopying)))
                .then_some(BOTTOM_COPY_DECISION_TIMEOUT);
            }
            ActionOutcome::BottomCopyBuryComplete { .. } => {
                self.bottom_copy_remaining = self
                    .game
                    .as_ref()
                    .is_some_and(|game| matches!(game.phase(), Phase::BottomCopying))
                    .then_some(BOTTOM_COPY_DECISION_TIMEOUT);
            }
            ActionOutcome::ThrowFailed {
                player,
                attempted,
                forced,
                penalty_points,
                ..
            } => {
                self.throw_penalties[usize::from(player.0)] = *penalty_points;
                self.held_throw_failure = Some(HeldThrowFailure {
                    player: *player,
                    attempted: attempted.clone(),
                    forced: forced.clone(),
                    penalty_points: *penalty_points,
                    stage: ShengjiThrowFailureStage::Showing,
                    remaining: THROW_FAILURE_SHOW_DURATION,
                });
            }
            ActionOutcome::TrickComplete(trick) => {
                self.held_trick = Some((trick.clone(), TRICK_HOLD_DURATION));
            }
            ActionOutcome::HandComplete(result) => {
                self.held_trick = self
                    .game
                    .as_ref()
                    .and_then(|game| game.history().last().cloned())
                    .map(|trick| (trick, TRICK_HOLD_DURATION));
                self.teams.levels = result.levels;
                self.next_dealer = Some(result.next_dealer);
                self.apply_finished_reference_points(result);
                self.room.prepare_rematch();
            }
            _ => {}
        }
    }

    fn apply_finished_reference_points(&mut self, result: &HandResult) {
        if self.finished_reference_changes.is_some() {
            return;
        }
        let magnitude = finished_reference_point_magnitude(result);
        let hand_profile_stats = self.hand_profile_stats.clone();
        let bottom_burier = self.game.as_ref().and_then(GameState::bottom_burier);
        let mut changes = Vec::with_capacity(RuleSet::PLAYER_COUNT);
        for participant in &mut self.room.players {
            if usize::from(participant.id.0) >= RuleSet::PLAYER_COUNT {
                continue;
            }
            let team = leocard_shengji::TeamId(participant.id.0 % 2);
            let delta = if team == result.promoted_team {
                magnitude
            } else {
                -magnitude
            };
            participant.reference_points = participant
                .reference_points
                .saturating_add(i32::from(delta));
            participant.completed_games = participant.completed_games.saturating_add(1);
            let aggregate = participant
                .game_profiles
                .shengji
                .get_or_insert_with(ShengjiProfileStats::default);
            aggregate.completed_games = aggregate.completed_games.saturating_add(1);
            aggregate.total_reference_delta = aggregate
                .total_reference_delta
                .saturating_add(i64::from(delta));
            if team == result.dealer_team {
                aggregate.dealer_team_games = aggregate.dealer_team_games.saturating_add(1);
                aggregate.dealer_team_score = aggregate
                    .dealer_team_score
                    .saturating_add(u64::from(result.collecting_score));
                if result.kitty_multiplier == 0 {
                    aggregate.defended_kitty_games =
                        aggregate.defended_kitty_games.saturating_add(1);
                }
            } else {
                aggregate.collecting_team_games = aggregate.collecting_team_games.saturating_add(1);
                aggregate.collecting_team_score = aggregate
                    .collecting_team_score
                    .saturating_add(u64::from(result.collecting_score));
                if result.kitty_multiplier > 0 {
                    aggregate.captured_kitty_games =
                        aggregate.captured_kitty_games.saturating_add(1);
                }
            }
            if to_core_player(participant.id) == result.dealer {
                aggregate.dealer_games = aggregate.dealer_games.saturating_add(1);
            }
            if bottom_burier == Some(to_core_player(participant.id)) {
                aggregate.buried_games = aggregate.buried_games.saturating_add(1);
                aggregate.buried_points = aggregate
                    .buried_points
                    .saturating_add(u64::from(result.kitty_points));
            }
            if let Some(current) = hand_profile_stats.get(usize::from(participant.id.0)) {
                merge_shengji_profile_stats(aggregate, current);
            }
            changes.push(PlayerReferenceChange {
                player: participant.id,
                profile_id: participant.profile_id,
                delta,
            });
        }
        self.finished_settlement_id = Some(new_match_id());
        self.finished_reference_changes = Some(changes);
    }

    fn hand_result_view(&self, result: &HandResult) -> ShengjiHandResultView {
        ShengjiHandResultView {
            settlement_id: self
                .finished_settlement_id
                .expect("完成的小局已经生成独立结算标识"),
            dealer: from_core_player(result.dealer),
            dealer_team: result.dealer_team,
            collecting_team: result.collecting_team,
            trick_points: result.trick_points,
            penalty_adjustment: result.penalty_adjustment,
            kitty_points: result.kitty_points,
            kitty_multiplier: result.kitty_multiplier,
            collecting_score: result.collecting_score,
            promoted_team: result.promoted_team,
            promoted_steps: result.promoted_steps,
            next_dealer: from_core_player(result.next_dealer),
            levels: result.levels,
            reference_changes: self
                .finished_reference_changes
                .clone()
                .expect("完成的小局已经计算平台积分变化"),
        }
    }

    fn reject_game_error(
        &self,
        connection: ConnectionId,
        request_id: RequestId,
        error: GameError,
    ) -> Vec<Delivery> {
        self.room.reject(
            connection,
            request_id,
            RejectReason::GameViolation(GameViolation::Shengji(game_violation(error))),
        )
    }

    fn broadcast_events(&self, events: Vec<ShengjiEvent>) -> Vec<Delivery> {
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
                            ServerEvent::GameEvent(GameEvent::Shengji(event.clone())),
                        )
                    })
            })
            .collect()
    }
}

fn record_shengji_component(stats: &mut ShengjiProfileStats, component: &Component) {
    let index = match component {
        Component::Single { .. } | Component::Pair { .. } | Component::Triple { .. } => return,
        Component::Tractor { pair_count, .. } => {
            stats.longest_tractor = stats.longest_tractor.max(u16::from(*pair_count));
            0
        }
        Component::Titanic { triple_count, .. } => {
            stats.longest_titanic = stats.longest_titanic.max(u16::from(*triple_count));
            1
        }
        Component::Quad { .. } => 2,
        Component::Spaceship { quad_count, .. } => {
            stats.longest_space_fortress = stats.longest_space_fortress.max(u16::from(*quad_count));
            3
        }
    };
    stats.play_category_counts[index] = stats.play_category_counts[index].saturating_add(1);
}

fn merge_shengji_profile_stats(aggregate: &mut ShengjiProfileStats, current: &ShengjiProfileStats) {
    aggregate.declaration_games = aggregate
        .declaration_games
        .saturating_add(current.declaration_games);
    aggregate.counter_games = aggregate
        .counter_games
        .saturating_add(current.counter_games);
    aggregate.plays = aggregate.plays.saturating_add(current.plays);
    aggregate.winning_plays = aggregate
        .winning_plays
        .saturating_add(current.winning_plays);
    aggregate.crossing_games = aggregate
        .crossing_games
        .saturating_add(current.crossing_games);
    for (aggregate, current) in aggregate
        .play_category_counts
        .iter_mut()
        .zip(current.play_category_counts)
    {
        *aggregate = aggregate.saturating_add(current);
    }
    aggregate.longest_tractor = aggregate.longest_tractor.max(current.longest_tractor);
    aggregate.longest_titanic = aggregate.longest_titanic.max(current.longest_titanic);
    aggregate.longest_space_fortress = aggregate
        .longest_space_fortress
        .max(current.longest_space_fortress);
    aggregate.longest_throw = aggregate.longest_throw.max(current.longest_throw);
}

fn completed_trick_events(trick: &TrickRecord, collecting_score: u32) -> Vec<ShengjiEvent> {
    let last = trick
        .plays
        .last()
        .map(|(player, play)| ShengjiEvent::CardsPlayed {
            play: ShengjiPublicPlay {
                player: from_core_player(*player),
                play: play.clone(),
                throw_penalty: 0,
            },
            is_lead: false,
        });
    last.into_iter()
        .chain([ShengjiEvent::TrickFinished {
            winner: from_core_player(trick.winner),
            points: trick.points,
            collecting_score,
        }])
        .collect()
}

fn bottom_flip_reveal_view(reveal: &BottomFlipReveal) -> ShengjiBottomFlipRevealView {
    ShengjiBottomFlipRevealView {
        card: reveal.card,
        matches: reveal
            .matches
            .iter()
            .map(|matched| ShengjiBottomFlipMatchView {
                player: from_core_player(matched.player),
                cards: matched.cards.clone(),
            })
            .collect(),
        dealer: reveal.dealer.map(from_core_player),
    }
}

fn pre_kitty_collecting_score(result: &HandResult) -> u32 {
    i64::from(result.trick_points)
        .saturating_add(i64::from(result.penalty_adjustment))
        .max(0) as u32
}

fn finished_reference_point_magnitude(result: &HandResult) -> i16 {
    if result.promoted_team == result.dealer_team {
        i16::from(result.promoted_steps) * 2
    } else {
        i16::from(result.promoted_steps.saturating_add(1)) * 2
    }
}

fn game_violation(error: GameError) -> ShengjiViolation {
    match error {
        GameError::InvalidPlayer(_) => ShengjiViolation::InvalidPlayer,
        GameError::WrongPhase | GameError::RedealRequired | GameError::Rules(_) => {
            ShengjiViolation::WrongPhase
        }
        GameError::NotDealer => ShengjiViolation::NotDealer,
        GameError::WrongBuryCount { expected, actual } => ShengjiViolation::WrongBuryCount {
            expected: expected.min(usize::from(u8::MAX)) as u8,
            actual: actual.min(usize::from(u8::MAX)) as u8,
        },
        GameError::CardsNotOwned => ShengjiViolation::CardsNotOwned,
        GameError::CrossingNotEligible => ShengjiViolation::CrossingNotEligible,
        GameError::CrossingAlreadyDecided => ShengjiViolation::CrossingAlreadyDecided,
        GameError::WrongCrossingCount { expected, actual } => {
            ShengjiViolation::WrongCrossingCount {
                expected: expected.min(usize::from(u8::MAX)) as u8,
                actual: actual.min(usize::from(u8::MAX)) as u8,
            }
        }
        GameError::CrossingMustIncludeAllTrumps => ShengjiViolation::CrossingMustIncludeAllTrumps,
        GameError::CrossingReturnNotRequired => ShengjiViolation::CrossingReturnNotRequired,
        GameError::CrossingAlreadyReturned => ShengjiViolation::CrossingAlreadyReturned,
        GameError::NotBottomCopyPlayer => ShengjiViolation::NotBottomCopyPlayer,
        GameError::NotPlayersTurn { .. } => ShengjiViolation::NotPlayersTurn,
        GameError::Bid(error) => match error {
            BidError::InvalidPlayer(_) => ShengjiViolation::InvalidPlayer,
            BidError::Closed | BidError::NoDeclaration | BidError::InvalidCards => {
                ShengjiViolation::InvalidDeclaration
            }
            BidError::CardsNotOwned => ShengjiViolation::DeclarationCardsNotOwned,
            BidError::CounterRequiresPair => ShengjiViolation::CounterRequiresPair,
            BidError::NotStronger => ShengjiViolation::CounterNotStronger,
            BidError::ProtectedSuit => ShengjiViolation::ProtectedSuitCanOnlyBeCounteredByNoTrump,
            BidError::JokerRequired => ShengjiViolation::DeclarationRequiresJoker,
            BidError::NoTrumpCannotOpen => ShengjiViolation::NoTrumpCannotOpen,
        },
        GameError::Play(error) => play_violation(error),
        GameError::Follow(error) => match error {
            FollowError::Play(error) => play_violation(error),
            FollowError::WrongCardCount { expected, actual } => ShengjiViolation::WrongCardCount {
                expected: expected.min(usize::from(u8::MAX)) as u8,
                actual: actual.min(usize::from(u8::MAX)) as u8,
            },
            FollowError::MustFollowCategory { .. } => ShengjiViolation::MustFollowCategory,
            FollowError::MustFollowStructure => ShengjiViolation::MustFollowStructure,
        },
        GameError::InvalidDeckSize { .. } | GameError::InvalidDeckContents => {
            ShengjiViolation::WrongPhase
        }
    }
}

fn play_violation(error: PlayError) -> ShengjiViolation {
    match error {
        PlayError::Empty => ShengjiViolation::MustLeadWithCards,
        PlayError::DuplicatePhysicalCard | PlayError::CardsNotOwned => {
            ShengjiViolation::CardsNotOwned
        }
        PlayError::MixedCategory => ShengjiViolation::InvalidPattern,
        PlayError::ThrowDisabled => ShengjiViolation::ThrowDisabled,
    }
}

fn card_sort_key(card: Card, trump: leocard_shengji::Trump) -> (bool, u8, u8, u8) {
    (
        trump.is_trump(card),
        trump.strength(card),
        card.suit().map_or(4, leocard_shengji::Suit::bid_strength),
        card.deck(),
    )
}

fn validate_deck(deck: &[Card], rules: RuleSet) -> Result<(), HostError> {
    let expected = build_deck_for(rules.deck_count);
    if deck.len() != expected.len() {
        return Err(HostError::InvalidDeckSize {
            expected: expected.len(),
            actual: deck.len(),
        });
    }
    let expected = expected.into_iter().collect::<HashSet<_>>();
    let actual = deck.iter().copied().collect::<HashSet<_>>();
    if actual.len() != deck.len() || actual != expected {
        return Err(HostError::InvalidDeckContents);
    }
    Ok(())
}

fn shuffled_deck(rules: RuleSet) -> Vec<Card> {
    let mut deck = build_deck_for(rules.deck_count);
    fastrand::shuffle(&mut deck);
    deck
}

const fn to_core_player(player: PlayerId) -> CorePlayerId {
    CorePlayerId(player.0)
}

const fn from_core_player(player: CorePlayerId) -> PlayerId {
    PlayerId(player.0)
}

#[cfg(test)]
mod tests {
    use ed25519_dalek::{Signer, SigningKey};
    use leocard_protocol::{
        ClientCommand, GameCommand, ProfileId, ReconnectToken, SeatId, ServerEvent,
        join_identity_payload,
    };
    use leocard_shengji::{Category, Rank, Suit, build_deck_for};

    use super::*;

    const ROOM: RoomId = RoomId(8080);

    fn message(connection_index: u8, request: u64, command: ClientCommand) -> ClientMessage {
        let _ = connection_index;
        ClientMessage::new(ROOM, RequestId(request), command)
    }

    fn join_command(index: u8) -> ClientCommand {
        let name = format!("玩家{index}");
        let token = ReconnectToken(u64::from(index) + 1);
        let mut secret = [0; 32];
        secret[0] = index + 1;
        let key = SigningKey::from_bytes(&secret);
        let game_profiles = leocard_protocol::PlayerGameProfiles::default();
        let signature = key
            .sign(&join_identity_payload(
                ROOM,
                token,
                &name,
                0,
                0,
                &game_profiles,
            ))
            .to_bytes()
            .to_vec();
        ClientCommand::Join {
            name,
            reconnect_token: token,
            profile_id: ProfileId(key.verifying_key().to_bytes()),
            reference_points: 0,
            completed_games: 0,
            game_profiles,
            identity_signature: signature,
        }
    }

    fn started_session() -> (ShengjiSession, [ConnectionId; 4], Card) {
        started_session_with_rules(RuleSet::default())
    }

    fn started_session_with_rules(rules: RuleSet) -> (ShengjiSession, [ConnectionId; 4], Card) {
        let target = Card::suited(0, Suit::Diamond, Rank::Two);
        let mut deck = build_deck();
        let index = deck.iter().position(|card| *card == target).unwrap();
        deck.swap(0, index);
        let (session, connections) = started_session_with_deck(rules, deck);
        (session, connections, target)
    }

    fn started_session_with_deck(
        rules: RuleSet,
        deck: Vec<Card>,
    ) -> (ShengjiSession, [ConnectionId; 4]) {
        let mut session = ShengjiSession::new(ROOM, rules, deck).unwrap();
        let connections = [
            ConnectionId(10),
            ConnectionId(20),
            ConnectionId(30),
            ConnectionId(40),
        ];
        for (index, connection) in connections.into_iter().enumerate() {
            session.handle(
                connection,
                message(index as u8, 1, join_command(index as u8)),
            );
            session.handle(
                connection,
                message(
                    index as u8,
                    2,
                    ClientCommand::SelectSeat {
                        seat: SeatId(index as u8),
                    },
                ),
            );
            session.handle(
                connection,
                message(index as u8, 3, ClientCommand::SetReady { ready: true }),
            );
        }
        session.handle(connections[0], message(0, 4, ClientCommand::StartGame));
        (session, connections)
    }

    #[test]
    fn bottom_copy_is_public_keeps_the_dealer_and_times_out_as_a_pass() {
        let rules = RuleSet {
            bottom_copy: true,
            bottom_flip: true,
            ..RuleSet::default()
        };
        let diamond = Card::suited(0, Suit::Diamond, Rank::Two);
        let hearts = [
            Card::suited(0, Suit::Heart, Rank::Two),
            Card::suited(1, Suit::Heart, Rank::Two),
        ];
        let arranged = [
            (0, diamond),
            (2, Card::suited(1, Suit::Diamond, Rank::Two)),
            (4, Card::suited(0, Suit::Club, Rank::Two)),
            (6, Card::suited(1, Suit::Club, Rank::Two)),
            (1, hearts[0]),
            (5, hearts[1]),
            (8, Card::suited(0, Suit::Spade, Rank::Two)),
            (10, Card::suited(1, Suit::Spade, Rank::Two)),
            (12, Card::small_joker(0)),
            (14, Card::small_joker(1)),
            (17, Card::big_joker(0)),
            (19, Card::big_joker(1)),
        ];
        let mut deck = build_deck();
        for (target_index, card) in arranged {
            let index = deck
                .iter()
                .position(|candidate| *candidate == card)
                .unwrap();
            deck.swap(target_index, index);
        }
        let (mut session, connections) = started_session_with_deck(rules, deck);
        session.advance_time(DEAL_INTERVAL);
        session.handle(
            connections[0],
            message(
                0,
                5,
                ClientCommand::Game(GameCommand::Shengji(ShengjiCommand::Declare {
                    cards: vec![diamond],
                })),
            ),
        );
        session.advance_time(DEAL_INTERVAL * 99);
        session.advance_time(BIDDING_GRACE);
        let buried = session.game().unwrap().players()[0].hand[..rules.kitty_size()].to_vec();
        let inquiry = session.handle(
            connections[0],
            message(
                0,
                6,
                ClientCommand::Game(GameCommand::Shengji(ShengjiCommand::Bury {
                    cards: buried.clone(),
                })),
            ),
        );
        let first_burier = game_snapshot(&inquiry, connections[0]);
        assert_eq!(first_burier.your_buried, buried);
        assert!(
            connections[1..]
                .iter()
                .all(|connection| game_snapshot(&inquiry, *connection).your_buried.is_empty())
        );
        assert!(matches!(
            first_burier.phase,
            ShengjiPhaseView::BottomCopying {
                player: PlayerId(1),
                milliseconds_remaining: 10_000,
            }
        ));

        // 正常亮主定庄时，即使房间同时开启扳底配置，抄底仍然有效。
        let mut timeout_session = session.clone();
        let timeout = timeout_session.advance_time(BOTTOM_COPY_DECISION_TIMEOUT);
        assert!(matches!(
            game_snapshot(&timeout, connections[0]).phase,
            ShengjiPhaseView::Playing
        ));

        let copied = session.handle(
            connections[1],
            message(
                1,
                7,
                ClientCommand::Game(GameCommand::Shengji(ShengjiCommand::ChooseBottomCopy {
                    cards: Some(hearts.to_vec()),
                })),
            ),
        );
        let copier = game_snapshot(&copied, connections[1]);
        let opponent = game_snapshot(&copied, connections[2]);
        assert_eq!(copier.dealer, Some(PlayerId(0)));
        assert_eq!(copier.trump.unwrap().suit, Some(Suit::Heart));
        assert_eq!(copier.your_hand.len(), 33);
        assert!(copier.your_buried.is_empty());
        assert_eq!(opponent.your_hand.len(), 25);
        assert!(opponent.your_buried.is_empty());
        assert!(matches!(
            copier.phase,
            ShengjiPhaseView::BottomCopyBurying {
                player: PlayerId(1)
            }
        ));
        assert!(copied.iter().any(|delivery| matches!(
            &delivery.message.event,
            ServerEvent::GameEvent(GameEvent::Shengji(
                ShengjiEvent::BottomCopied { declaration }
            )) if declaration.player == PlayerId(1) && declaration.cards == hearts
        )));

        let reburied = session.game().unwrap().players()[1].hand[..rules.kitty_size()].to_vec();
        let finished = session.handle(
            connections[1],
            message(
                1,
                8,
                ClientCommand::Game(GameCommand::Shengji(ShengjiCommand::Bury {
                    cards: reburied.clone(),
                })),
            ),
        );
        let copier_after_bury = game_snapshot(&finished, connections[1]);
        assert_eq!(copier_after_bury.your_buried, reburied);
        assert!(
            connections
                .iter()
                .enumerate()
                .filter(|(index, _)| *index != 1)
                .all(|(_, connection)| game_snapshot(&finished, *connection)
                    .your_buried
                    .is_empty())
        );
        assert!(matches!(
            game_snapshot(&finished, connections[0]).phase,
            ShengjiPhaseView::Playing
        ));
        assert_eq!(session.game().unwrap().dealer(), Some(CorePlayerId(0)));
    }

    #[test]
    fn five_trump_crossing_commands_transfer_privately_and_only_once() {
        let rules = RuleSet {
            five_trump_crossing: true,
            ..RuleSet::default()
        };
        let (mut session, connections, target) = started_session_with_rules(rules);
        session.advance_time(DEAL_INTERVAL);
        session.handle(
            connections[0],
            message(
                0,
                5,
                ClientCommand::Game(GameCommand::Shengji(ShengjiCommand::Declare {
                    cards: vec![target],
                })),
            ),
        );
        session.advance_time(DEAL_INTERVAL * 99);
        session.advance_time(BIDDING_GRACE);

        let game = session.game().unwrap();
        let trump = game.trump().unwrap();
        let dealer_hand = game.players()[0].hand.clone();
        let trump_count = dealer_hand
            .iter()
            .filter(|card| trump.is_trump(**card))
            .count();
        let trump_to_bury = trump_count.saturating_sub(5);
        assert!(trump_to_bury <= rules.kitty_size());
        let mut buried = dealer_hand
            .iter()
            .copied()
            .filter(|card| trump.is_trump(*card))
            .take(trump_to_bury)
            .collect::<Vec<_>>();
        let side_cards_to_bury = rules.kitty_size() - buried.len();
        buried.extend(
            dealer_hand
                .iter()
                .copied()
                .filter(|card| !trump.is_trump(*card))
                .take(side_cards_to_bury),
        );
        assert_eq!(buried.len(), rules.kitty_size());
        let deliveries = session.handle(
            connections[0],
            message(
                0,
                6,
                ClientCommand::Game(GameCommand::Shengji(ShengjiCommand::Bury { cards: buried })),
            ),
        );
        let snapshot = game_snapshot(&deliveries, connections[0]);
        let ShengjiPhaseView::FiveTrumpCrossing { eligible, .. } = snapshot.phase else {
            panic!("有主且庄家只剩五张主牌时应进入五主过江阶段");
        };
        assert!(eligible.contains(&PlayerId(0)));
        let mut waiting_for_decision = session.clone();
        assert!(
            waiting_for_decision
                .advance_time(Duration::from_secs(90))
                .is_empty()
        );
        assert!(matches!(
            waiting_for_decision.game().unwrap().phase(),
            Phase::FiveTrumpCrossing
        ));

        let hand = session.game().unwrap().players()[0].hand.clone();
        let mut outgoing = hand
            .iter()
            .copied()
            .filter(|card| trump.is_trump(*card))
            .collect::<Vec<_>>();
        let filler_count = 5 - outgoing.len();
        outgoing.extend(
            hand.iter()
                .copied()
                .filter(|card| !trump.is_trump(*card))
                .take(filler_count),
        );
        assert_eq!(outgoing.len(), 5);
        let mut final_decision = session.handle(
            connections[0],
            message(
                0,
                7,
                ClientCommand::Game(GameCommand::Shengji(
                    ShengjiCommand::ChooseFiveTrumpCrossing {
                        cards: Some(outgoing.clone()),
                    },
                )),
            ),
        );
        for player in eligible.into_iter().filter(|player| *player != PlayerId(0)) {
            final_decision = session.handle(
                connections[usize::from(player.0)],
                message(
                    player.0,
                    10,
                    ClientCommand::Game(GameCommand::Shengji(
                        ShengjiCommand::ChooseFiveTrumpCrossing { cards: None },
                    )),
                ),
            );
        }
        let partner = game_snapshot(&final_decision, connections[2]);
        assert!(outgoing.iter().all(|card| partner.your_hand.contains(card)));
        let opponent = game_snapshot(&final_decision, connections[1]);
        assert!(
            outgoing
                .iter()
                .any(|card| !opponent.your_hand.contains(card))
        );
        assert!(matches!(
            partner.phase,
            ShengjiPhaseView::FiveTrumpCrossing {
                stage: ShengjiFiveTrumpCrossingStage::Returning,
                ..
            }
        ));

        let mut waiting_for_return = session.clone();
        assert!(
            waiting_for_return
                .advance_time(Duration::from_secs(90))
                .is_empty()
        );
        assert!(matches!(
            waiting_for_return.game().unwrap().phase(),
            Phase::FiveTrumpCrossing
        ));

        let deliveries = session.handle(
            connections[2],
            message(
                2,
                11,
                ClientCommand::Game(GameCommand::Shengji(
                    ShengjiCommand::ReturnFiveTrumpCrossing {
                        cards: outgoing.clone(),
                    },
                )),
            ),
        );
        assert!(matches!(
            game_snapshot(&deliveries, connections[0]).phase,
            ShengjiPhaseView::Playing
        ));
        let rejected = session.handle(
            connections[0],
            message(
                0,
                12,
                ClientCommand::Game(GameCommand::Shengji(
                    ShengjiCommand::ChooseFiveTrumpCrossing { cards: None },
                )),
            ),
        );
        assert!(rejected.iter().any(|delivery| matches!(
            delivery.message.event,
            ServerEvent::Rejected {
                reason: RejectReason::GameViolation(GameViolation::Shengji(
                    ShengjiViolation::WrongPhase
                ))
            }
        )));
    }

    #[test]
    fn power_outage_opens_ten_second_round_then_bottom_flip_is_publicly_held() {
        let rules = RuleSet {
            power_outage_dealer: true,
            bottom_flip: true,
            ..RuleSet::default()
        };
        let (mut session, connections, _) = started_session_with_rules(rules);
        let deliveries = session.advance_time(DEAL_INTERVAL * 100);
        let before = game_snapshot(&deliveries, connections[0]);
        assert!(matches!(
            before.phase,
            ShengjiPhaseView::BiddingGrace {
                power_outage: false,
                ..
            }
        ));
        let hand_lengths = before
            .players
            .iter()
            .map(|player| player.hand_len)
            .collect::<Vec<_>>();

        let deliveries = session.advance_time(BIDDING_GRACE);
        let outage = game_snapshot(&deliveries, connections[0]);
        assert_eq!(outage.dealer, Some(PlayerId(1)));
        assert_eq!(outage.bidding_level, Rank::Two);
        assert!(matches!(
            outage.phase,
            ShengjiPhaseView::BiddingGrace {
                milliseconds_remaining: 10_000,
                power_outage: true,
                ..
            }
        ));
        assert_eq!(
            outage
                .players
                .iter()
                .map(|player| player.hand_len)
                .collect::<Vec<_>>(),
            hand_lengths
        );
        assert!(
            session
                .room
                .players
                .iter()
                .all(|player| player.reference_points == 0)
        );

        let deliveries = session.advance_time(POWER_OUTAGE_BIDDING_GRACE);
        assert!(matches!(
            game_snapshot(&deliveries, connections[0]).phase,
            ShengjiPhaseView::BottomFlipping { reveal: None }
        ));
        let deliveries = session.advance_time(BOTTOM_FLIP_START_DELAY);
        let reveal = game_snapshot(&deliveries, connections[0]);
        let ShengjiPhaseView::BottomFlipping {
            reveal: Some(reveal_view),
        } = reveal.phase
        else {
            panic!("扳底应公开当前底牌及所有同牌");
        };
        assert_eq!(reveal_view.card, Card::suited(1, Suit::Spade, Rank::Nine));
        assert_eq!(reveal_view.dealer, Some(PlayerId(2)));
        assert_eq!(reveal_view.matches.len(), 1);
        assert_eq!(reveal_view.matches[0].player, PlayerId(2));
        assert_eq!(reveal.dealer, Some(PlayerId(2)));
        assert_eq!(reveal.trump.unwrap().suit, Some(Suit::Spade));

        // 旧的 1.4 秒节奏会在玩家交互演出完成前收起翻牌；
        // 现在到这个时点仍必须保持公开。
        session.advance_time(Duration::from_millis(1400));
        assert!(matches!(
            session.game_snapshot(PlayerId(0)).phase,
            ShengjiPhaseView::BottomFlipping { reveal: Some(_) }
        ));

        let deliveries =
            session.advance_time(BOTTOM_FLIP_HOLD_DURATION - Duration::from_millis(1400));
        assert!(matches!(
            game_snapshot(&deliveries, connections[0]).phase,
            ShengjiPhaseView::Burying
        ));
    }

    #[test]
    fn three_deck_session_requires_and_accepts_the_rule_selected_deck() {
        let rules = RuleSet {
            deck_count: 3,
            ..RuleSet::default()
        };
        assert_eq!(
            ShengjiSession::new(ROOM, rules, build_deck()).unwrap_err(),
            HostError::InvalidDeckSize {
                expected: 162,
                actual: 108,
            }
        );
        let session = ShengjiSession::new(ROOM, rules, build_deck_for(3)).unwrap();
        assert_eq!(session.rules().deck_count, 3);
    }

    #[test]
    fn four_deck_session_requires_and_accepts_the_rule_selected_deck() {
        let rules = RuleSet {
            deck_count: 4,
            ..RuleSet::default()
        };
        assert_eq!(
            ShengjiSession::new(ROOM, rules, build_deck_for(3)).unwrap_err(),
            HostError::InvalidDeckSize {
                expected: 216,
                actual: 162,
            }
        );
        let session = ShengjiSession::new(ROOM, rules, build_deck_for(4)).unwrap();
        assert_eq!(session.rules().deck_count, 4);
    }

    fn game_snapshot(deliveries: &[Delivery], recipient: ConnectionId) -> ShengjiSnapshot {
        deliveries
            .iter()
            .rev()
            .find_map(|delivery| {
                if delivery.recipient != recipient {
                    return None;
                }
                let ServerEvent::GameSnapshot(GameSnapshot::Shengji(snapshot)) =
                    &delivery.message.event
                else {
                    return None;
                };
                Some(snapshot.clone())
            })
            .expect("recipient receives a Shengji snapshot")
    }

    #[test]
    fn dealing_is_clocked_and_each_snapshot_keeps_other_hands_private() {
        let (mut session, connections, target) = started_session();
        assert!(
            session
                .advance_time(DEAL_INTERVAL - Duration::from_millis(1))
                .is_empty()
        );
        let deliveries = session.advance_time(Duration::from_millis(1));
        let host = game_snapshot(&deliveries, connections[0]);
        let guest = game_snapshot(&deliveries, connections[1]);
        assert_eq!(host.your_hand, vec![target]);
        assert!(guest.your_hand.is_empty());
        assert_eq!(host.players[0].hand_len, 1);
        assert!(matches!(
            host.phase,
            ShengjiPhaseView::Dealing {
                cards_remaining: 99
            }
        ));
    }

    #[test]
    fn bid_pass_confirmations_reset_on_a_new_declaration_and_close_early_when_unanimous() {
        let target = Card::suited(0, Suit::Heart, Rank::Two);
        let mut deck = build_deck();
        let index = deck.iter().position(|card| *card == target).unwrap();
        deck.swap(1, index);
        let (mut session, connections) = started_session_with_deck(RuleSet::default(), deck);
        session.advance_time(DEAL_INTERVAL * 100);

        let first_confirmation = session.handle(
            connections[0],
            message(
                0,
                5,
                ClientCommand::Game(GameCommand::Shengji(ShengjiCommand::ConfirmBidPass)),
            ),
        );
        assert!(matches!(
            game_snapshot(&first_confirmation, connections[0]).phase,
            ShengjiPhaseView::BiddingGrace {
                confirmed_count: 1,
                you_confirmed: true,
                ..
            }
        ));
        assert!(matches!(
            game_snapshot(&first_confirmation, connections[1]).phase,
            ShengjiPhaseView::BiddingGrace {
                confirmed_count: 1,
                you_confirmed: false,
                ..
            }
        ));

        let declaration = session.handle(
            connections[1],
            message(
                1,
                5,
                ClientCommand::Game(GameCommand::Shengji(ShengjiCommand::Declare {
                    cards: vec![target],
                })),
            ),
        );
        assert!(matches!(
            game_snapshot(&declaration, connections[0]).phase,
            ShengjiPhaseView::BiddingGrace {
                confirmed_count: 0,
                you_confirmed: false,
                ..
            }
        ));

        for (index, connection) in connections[..3].iter().copied().enumerate() {
            let deliveries = session.handle(
                connection,
                message(
                    index as u8,
                    6,
                    ClientCommand::Game(GameCommand::Shengji(ShengjiCommand::ConfirmBidPass)),
                ),
            );
            assert!(matches!(
                game_snapshot(&deliveries, connections[3]).phase,
                ShengjiPhaseView::BiddingGrace { .. }
            ));
        }
        let locked = session.handle(
            connections[3],
            message(
                3,
                6,
                ClientCommand::Game(GameCommand::Shengji(ShengjiCommand::ConfirmBidPass)),
            ),
        );
        assert!(matches!(
            game_snapshot(&locked, connections[3]).phase,
            ShengjiPhaseView::Burying
        ));
        assert!(locked.iter().any(|delivery| matches!(
            delivery.message.event,
            ServerEvent::GameEvent(GameEvent::Shengji(ShengjiEvent::BiddingLocked {
                declaration: Some(_)
            }))
        )));
    }

    #[test]
    fn shengji_player_ids_follow_seat_order_before_teams_are_formed() {
        let mut session = ShengjiSession::new(ROOM, RuleSet::default(), build_deck()).unwrap();
        let connections = [
            ConnectionId(10),
            ConnectionId(20),
            ConnectionId(30),
            ConnectionId(40),
        ];
        let seats = [SeatId(2), SeatId(0), SeatId(3), SeatId(1)];
        for (index, connection) in connections.into_iter().enumerate() {
            session.handle(
                connection,
                message(index as u8, 1, join_command(index as u8)),
            );
            session.handle(
                connection,
                message(
                    index as u8,
                    2,
                    ClientCommand::SelectSeat { seat: seats[index] },
                ),
            );
            session.handle(
                connection,
                message(index as u8, 3, ClientCommand::SetReady { ready: true }),
            );
        }
        session.handle(connections[0], message(0, 4, ClientCommand::StartGame));

        for player in &session.room.players {
            assert_eq!(player.id.0, player.seat.unwrap().0);
        }
        assert_eq!(session.room.player_id(connections[1]), Some(PlayerId(0)));
        assert_eq!(session.room.player_id(connections[3]), Some(PlayerId(1)));
        assert_eq!(session.room.player_id(connections[0]), Some(PlayerId(2)));
        assert_eq!(session.room.player_id(connections[2]), Some(PlayerId(3)));
    }

    #[test]
    fn declaration_grace_closes_into_private_kitty_and_burying() {
        let (mut session, connections, target) = started_session();
        session.advance_time(DEAL_INTERVAL);
        let declaration = session.handle(
            connections[0],
            message(
                0,
                5,
                ClientCommand::Game(GameCommand::Shengji(ShengjiCommand::Declare {
                    cards: vec![target],
                })),
            ),
        );
        assert!(matches!(
            game_snapshot(&declaration, connections[0]).declaration,
            Some(ShengjiDeclarationView {
                player: PlayerId(0),
                ..
            })
        ));
        assert_eq!(
            game_snapshot(&declaration, connections[0]).dealer,
            Some(PlayerId(0))
        );
        assert_eq!(
            game_snapshot(&declaration, connections[0]).your_exposed_cards,
            vec![target]
        );
        assert!(
            game_snapshot(&declaration, connections[1])
                .your_exposed_cards
                .is_empty()
        );

        let deliveries = session.advance_time(DEAL_INTERVAL * 99);
        assert!(matches!(
            game_snapshot(&deliveries, connections[0]).phase,
            ShengjiPhaseView::BiddingGrace { .. }
        ));
        let deliveries = session.advance_time(BIDDING_GRACE);
        let dealer = game_snapshot(&deliveries, connections[0]);
        let guest = game_snapshot(&deliveries, connections[1]);
        assert_eq!(dealer.your_hand.len(), 33);
        assert_eq!(guest.your_hand.len(), 25);
        assert_eq!(dealer.buried_count, 0);
        assert!(matches!(dealer.phase, ShengjiPhaseView::Burying));
    }

    #[test]
    fn first_hand_dealer_badge_follows_initial_bid_and_counter_immediately() {
        let diamond = Card::suited(0, Suit::Diamond, Rank::Two);
        let spades = [
            Card::suited(0, Suit::Spade, Rank::Two),
            Card::suited(1, Suit::Spade, Rank::Two),
        ];
        let mut deck = build_deck();
        for (target_index, card) in [(0, diamond), (1, spades[0]), (5, spades[1])] {
            let source_index = deck
                .iter()
                .position(|candidate| *candidate == card)
                .unwrap();
            deck.swap(target_index, source_index);
        }
        let (mut session, connections) = started_session_with_deck(RuleSet::default(), deck);
        session.advance_time(DEAL_INTERVAL * 6);

        let initial = session.handle(
            connections[0],
            message(
                0,
                5,
                ClientCommand::Game(GameCommand::Shengji(ShengjiCommand::Declare {
                    cards: vec![diamond],
                })),
            ),
        );
        assert_eq!(
            game_snapshot(&initial, connections[0]).dealer,
            Some(PlayerId(0))
        );

        let counter = session.handle(
            connections[1],
            message(
                1,
                6,
                ClientCommand::Game(GameCommand::Shengji(ShengjiCommand::Declare {
                    cards: spades.to_vec(),
                })),
            ),
        );
        assert_eq!(
            game_snapshot(&counter, connections[0]).dealer,
            Some(PlayerId(1))
        );
    }

    #[test]
    fn later_hand_snapshot_exposes_the_fixed_dealer_while_dealing() {
        let (mut session, _, _) = started_session();
        session.next_dealer = Some(CorePlayerId(2));
        session.start_hand(true).unwrap();

        let snapshot = session.game_snapshot(PlayerId(0));
        assert!(matches!(snapshot.phase, ShengjiPhaseView::Dealing { .. }));
        assert_eq!(snapshot.dealer, Some(PlayerId(2)));
    }

    #[test]
    fn fifth_and_sixth_seats_are_invalid_in_a_shengji_room() {
        let mut session = ShengjiSession::new(ROOM, RuleSet::default(), build_deck()).unwrap();
        let connection = ConnectionId(10);
        session.handle(connection, message(0, 1, join_command(0)));
        let deliveries = session.handle(
            connection,
            message(0, 2, ClientCommand::SelectSeat { seat: SeatId(4) }),
        );
        assert!(deliveries.iter().any(|delivery| matches!(
            delivery.message.event,
            ServerEvent::Rejected {
                reason: RejectReason::InvalidSeat
            }
        )));
    }

    #[test]
    fn auto_play_can_be_toggled_for_a_started_shengji_player() {
        let (mut session, connections, _) = started_session();
        let deliveries = session.handle(
            connections[0],
            message(
                0,
                5,
                ClientCommand::Game(GameCommand::Shengji(ShengjiCommand::SetAutoPlay {
                    enabled: true,
                })),
            ),
        );
        let snapshot = game_snapshot(&deliveries, connections[0]);
        assert!(
            snapshot
                .players
                .iter()
                .find(|player| player.id == PlayerId(0))
                .unwrap()
                .auto_play
        );
    }

    #[test]
    fn completed_trick_remains_visible_for_the_hold_duration() {
        let (mut session, connections, _) = started_session();
        let trick = TrickRecord {
            leader: CorePlayerId(0),
            plays: Vec::new(),
            winner: CorePlayerId(0),
            points: 0,
        };
        session.after_game_outcome(&ActionOutcome::TrickComplete(trick));

        let held = session.game_snapshot(PlayerId(0));
        assert!(held.trick.is_some());
        assert_eq!(held.current_player, None);
        assert!(
            session
                .advance_time(TRICK_HOLD_DURATION - Duration::from_millis(1))
                .is_empty()
        );

        let deliveries = session.advance_time(Duration::from_millis(1));
        assert!(session.held_trick.is_none());
        assert!(game_snapshot(&deliveries, connections[0]).trick.is_none());
    }

    #[test]
    fn failed_throw_is_shown_then_returned_before_the_forced_play_is_revealed() {
        let (mut session, connections, target) = started_session();
        session.advance_time(DEAL_INTERVAL);
        session.handle(
            connections[0],
            message(
                0,
                5,
                ClientCommand::Game(GameCommand::Shengji(ShengjiCommand::Declare {
                    cards: vec![target],
                })),
            ),
        );
        session.advance_time(DEAL_INTERVAL * 99);
        session.advance_time(BIDDING_GRACE);
        let buried = session.game.as_ref().unwrap().players()[0].hand[..8].to_vec();
        session.handle(
            connections[0],
            message(
                0,
                6,
                ClientCommand::Game(GameCommand::Shengji(ShengjiCommand::Bury { cards: buried })),
            ),
        );

        let hand = &session.game.as_ref().unwrap().players()[0].hand;
        let forced_card = hand[0];
        let returned_card = hand[1];
        session
            .game
            .as_mut()
            .unwrap()
            .play_cards(CorePlayerId(0), &[forced_card])
            .unwrap();
        let forced = session
            .game
            .as_ref()
            .unwrap()
            .current_trick()
            .unwrap()
            .plays[0]
            .1
            .clone();
        session.after_game_outcome(&ActionOutcome::ThrowFailed {
            player: CorePlayerId(0),
            attempted: vec![forced_card, returned_card],
            forced: forced.clone(),
            penalty_points: 10,
            next: CorePlayerId(1),
        });

        let showing = session.game_snapshot(PlayerId(0));
        assert_eq!(showing.current_player, None);
        assert_eq!(
            showing.throw_failure,
            Some(ShengjiThrowFailureView {
                player: PlayerId(0),
                attempted: vec![forced_card, returned_card],
                forced: forced.clone(),
                penalty_points: 10,
                stage: ShengjiThrowFailureStage::Showing,
            })
        );
        let blocked = session.handle(
            connections[1],
            message(
                1,
                7,
                ClientCommand::Game(GameCommand::Shengji(ShengjiCommand::PlayCards {
                    cards: vec![session.game.as_ref().unwrap().players()[1].hand[0]],
                })),
            ),
        );
        assert!(blocked.iter().any(|delivery| matches!(
            delivery.message.event,
            ServerEvent::Rejected {
                reason: RejectReason::GameViolation(GameViolation::Shengji(
                    ShengjiViolation::WrongPhase
                ))
            }
        )));

        assert!(
            session
                .advance_time(THROW_FAILURE_SHOW_DURATION - Duration::from_millis(1))
                .is_empty()
        );
        let returning_deliveries = session.advance_time(Duration::from_millis(1));
        let returning = game_snapshot(&returning_deliveries, connections[0]);
        assert_eq!(
            returning.throw_failure.unwrap().stage,
            ShengjiThrowFailureStage::Returning
        );
        assert_eq!(returning.current_player, None);

        assert!(
            session
                .advance_time(THROW_FAILURE_RETURN_DURATION - Duration::from_millis(1))
                .is_empty()
        );
        let revealed_deliveries = session.advance_time(Duration::from_millis(1));
        assert!(revealed_deliveries.iter().any(|delivery| matches!(
            &delivery.message.event,
            ServerEvent::GameEvent(GameEvent::Shengji(ShengjiEvent::CardsPlayed {
                play,
                is_lead: true,
            })) if play.player == PlayerId(0)
                && play.play == forced
                && play.throw_penalty == 10
        )));
        let revealed = game_snapshot(&revealed_deliveries, connections[0]);
        assert_eq!(revealed.throw_failure, None);
        assert_eq!(revealed.current_player, Some(PlayerId(1)));
    }

    fn result_for_reference_points(
        dealer_wins: bool,
        promoted_steps: u8,
        collecting_score: u32,
    ) -> HandResult {
        let dealer_team = leocard_shengji::TeamId(0);
        HandResult {
            dealer: CorePlayerId(0),
            dealer_team,
            collecting_team: leocard_shengji::TeamId(1),
            trick_points: collecting_score,
            penalty_adjustment: 0,
            kitty_points: 0,
            kitty_multiplier: 0,
            collecting_score,
            promoted_team: if dealer_wins {
                dealer_team
            } else {
                leocard_shengji::TeamId(1)
            },
            promoted_steps,
            next_dealer: CorePlayerId(1),
            levels: [Rank::Two, Rank::Two],
        }
    }

    #[test]
    fn hand_rating_magnitude_matches_upgrade_outcome() {
        assert_eq!(
            finished_reference_point_magnitude(&result_for_reference_points(true, 3, 0)),
            6
        );
        assert_eq!(
            finished_reference_point_magnitude(&result_for_reference_points(true, 1, 40)),
            2
        );
        assert_eq!(
            finished_reference_point_magnitude(&result_for_reference_points(false, 0, 80)),
            2
        );
        assert_eq!(
            finished_reference_point_magnitude(&result_for_reference_points(false, 1, 120)),
            4
        );
        assert_eq!(
            finished_reference_point_magnitude(&result_for_reference_points(false, 2, 160)),
            6
        );
    }

    #[test]
    fn hand_rating_is_applied_once_and_updates_every_player() {
        let (mut session, _, _) = started_session();
        session.hand_profile_stats[0].declaration_games = 1;
        session.hand_profile_stats[0].counter_games = 1;
        session.hand_profile_stats[0].plays = 4;
        session.hand_profile_stats[0].winning_plays = 2;
        let result = result_for_reference_points(false, 1, 120);
        session.apply_finished_reference_points(&result);
        let settlement_id = session.finished_settlement_id;
        let changes = session.finished_reference_changes.clone().unwrap();

        assert!(settlement_id.is_some());
        assert_eq!(changes.len(), 4);
        for participant in &session.room.players {
            let expected = if participant.id.0 % 2 == 1 { 4 } else { -4 };
            assert_eq!(participant.reference_points, expected);
            assert_eq!(participant.completed_games, 1);
            let stats = participant.game_profiles.shengji.as_ref().unwrap();
            assert_eq!(stats.completed_games, 1);
            assert_eq!(stats.total_reference_delta, i64::from(expected));
            if participant.id.0 % 2 == 0 {
                assert_eq!(stats.dealer_team_games, 1);
                assert_eq!(stats.dealer_team_score, 120);
                assert_eq!(stats.defended_kitty_games, 1);
            } else {
                assert_eq!(stats.collecting_team_games, 1);
                assert_eq!(stats.collecting_team_score, 120);
                assert_eq!(stats.captured_kitty_games, 0);
            }
            assert_eq!(stats.dealer_games, u32::from(participant.id == PlayerId(0)));
            assert_eq!(
                changes
                    .iter()
                    .find(|change| change.player == participant.id)
                    .unwrap()
                    .delta,
                expected as i16
            );
        }
        let dealer_stats = session.room.players[0]
            .game_profiles
            .shengji
            .as_ref()
            .unwrap();
        assert_eq!(dealer_stats.declaration_games, 1);
        assert_eq!(dealer_stats.counter_games, 1);
        assert_eq!(dealer_stats.plays, 4);
        assert_eq!(dealer_stats.winning_plays, 2);

        session.apply_finished_reference_points(&result);
        assert_eq!(session.finished_settlement_id, settlement_id);
        assert!(
            session
                .room
                .players
                .iter()
                .all(|participant| participant.completed_games == 1)
        );
    }

    #[test]
    fn throw_profile_lengths_include_every_internal_sequence() {
        let (mut session, _, card) = started_session();
        let play = ClassifiedPlay {
            cards: vec![card; 26],
            category: Category::Suit(Suit::Diamond),
            components: vec![
                Component::Single { card, strength: 2 },
                Component::Pair {
                    cards: [card; 2],
                    strength: 3,
                },
                Component::Triple {
                    cards: [card; 3],
                    strength: 4,
                },
                Component::Tractor {
                    cards: vec![card; 6],
                    pair_count: 3,
                    top_strength: 8,
                },
                Component::Titanic {
                    cards: vec![card; 6],
                    triple_count: 2,
                    top_strength: 9,
                },
                Component::Spaceship {
                    cards: vec![card; 8],
                    quad_count: 2,
                    top_strength: 10,
                },
            ],
        };

        session.record_profile_play(CorePlayerId(0), &play, true, true);

        let stats = &session.hand_profile_stats[0];
        assert_eq!(stats.plays, 1);
        assert_eq!(stats.winning_plays, 1);
        assert_eq!(stats.play_category_counts[0], 1);
        assert_eq!(stats.play_category_counts[1], 1);
        assert_eq!(stats.play_category_counts[3], 1);
        assert_eq!(stats.play_category_counts[4], 1);
        assert_eq!(stats.play_category_counts.iter().sum::<u32>(), 4);
        assert_eq!(stats.longest_tractor, 3);
        assert_eq!(stats.longest_titanic, 2);
        assert_eq!(stats.longest_space_fortress, 2);
        assert_eq!(stats.longest_throw, 26);
    }

    #[cfg(feature = "developer")]
    #[test]
    fn developer_bots_bid_bury_and_play_with_the_greedy_policy() {
        let target = Card::suited(0, Suit::Diamond, Rank::Two);
        let mut deck = build_deck();
        let target_index = deck.iter().position(|card| *card == target).unwrap();
        deck.swap(1, target_index);
        let mut session = ShengjiSession::new(ROOM, RuleSet::default(), deck).unwrap();
        let host = ConnectionId(10);
        session.handle(host, message(0, 1, join_command(0)));
        session.handle(
            host,
            message(0, 2, ClientCommand::SelectSeat { seat: SeatId(0) }),
        );
        for seat in 1..PLAYER_COUNT {
            session.handle(
                host,
                message(
                    0,
                    u64::from(seat) + 2,
                    ClientCommand::ConfigureBotSeat {
                        seat: SeatId(seat),
                        occupied: true,
                    },
                ),
            );
        }
        session.handle(host, message(0, 6, ClientCommand::StartGame));
        let deliveries = session.advance_time(DEAL_INTERVAL * 100);
        let bidding = game_snapshot(&deliveries, host);
        assert_eq!(bidding.declaration.unwrap().player, PlayerId(1));

        session.advance_time(BIDDING_GRACE);
        session.advance_time(AUTOMATIC_ACTION_DELAY);
        let deliveries = session.advance_time(AUTOMATIC_ACTION_DELAY);
        let playing = game_snapshot(&deliveries, host);
        assert!(matches!(playing.phase, ShengjiPhaseView::Playing));
        assert_eq!(
            playing.trick.as_ref().map(|trick| trick.plays.len()),
            Some(1)
        );
        assert_eq!(playing.current_player, Some(PlayerId(2)));
    }
}
