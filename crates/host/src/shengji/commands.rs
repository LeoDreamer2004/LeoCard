use super::{ShengjiSession, shuffled_deck};
use crate::lifecycle::HostedGameLifecycle;
use crate::{ConnectionId, Delivery};
use leocard_game_common::PlayerId;
use leocard_protocol::{
    GameViolation, PlayerViolation, RejectReason, RequestId, RoomViolation, ShengjiEvent,
    ShengjiViolation,
};
use leocard_shengji::{
    ShengjiActionOutcome, ShengjiCard, ShengjiGameError, ShengjiGameState, ShengjiPhase,
    ShengjiRuleSet,
};
use std::time::Duration;

impl ShengjiSession {
    pub(super) fn update_rules(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        rules: ShengjiRuleSet,
    ) -> Vec<Delivery> {
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
                RejectReason::Room(RoomViolation::OnlyHostCanConfigure),
            );
        }
        let Ok(rules) = rules.validate() else {
            return self.room.reject(
                connection,
                request_id,
                RejectReason::Game(GameViolation::InvalidRuleConfiguration),
            );
        };
        if self
            .room
            .players
            .iter()
            .filter(|player| !player.left)
            .count()
            > usize::from(rules.player_count())
        {
            return self.room.reject(
                connection,
                request_id,
                RejectReason::Game(GameViolation::InvalidRuleConfiguration),
            );
        }
        if self.rules != rules {
            if self.rules.mode != rules.mode {
                self.room
                    .resize_seats(rules.player_count())
                    .expect("validated rules and player count fit the new seats");
            }
            self.rules = rules;
            self.room.reset_ready_after_rules_change();
            self.shuffled_deck = Some(shuffled_deck(self.rules));
            self.room.bump_revision();
        }
        self.broadcast_lobby(Some((connection, request_id)))
    }

    pub(super) fn start_hand(&mut self) -> Result<(), ShengjiGameError> {
        let deck = self
            .shuffled_deck
            .take()
            .unwrap_or_else(|| shuffled_deck(self.rules));
        let fixed_dealer = self.next_dealer;
        self.game = Some(ShengjiGameState::new(
            self.rules,
            self.teams.clone(),
            fixed_dealer,
            fixed_dealer.unwrap_or(PlayerId(0)),
            deck,
        )?);
        // 无人亮主重新发牌时也递增，让客户端重置逐张发牌动画。
        self.hand_number = self.hand_number.saturating_add(1);
        self.flow.deal_elapsed = Duration::ZERO;
        self.flow.bidding_remaining = None;
        self.flow.bid_pass_confirmed = [false; ShengjiRuleSet::MAX_PLAYERS as usize];
        self.flow.bottom_flip_reveal = None;
        self.flow.bottom_flip_remaining = None;
        self.flow.bottom_copy_remaining = None;
        self.flow.automatic_action = None;
        self.flow.redeal_remaining = None;
        self.presentation.throw_penalties = [0; ShengjiRuleSet::MAX_PLAYERS as usize];
        self.presentation.bottom_copy_decisions.clear();
        self.presentation.throw_failure = None;
        self.presentation.trick = None;
        self.statistics.start_hand(self.rules.player_count());
        Ok(())
    }

    pub(super) fn set_auto_play(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        enabled: bool,
    ) -> Vec<Delivery> {
        let Some(player) = self.room.player_id(connection) else {
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

    pub(super) fn declare(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        cards: Vec<ShengjiCard>,
    ) -> Vec<Delivery> {
        let Some(player) = self.room.player_id(connection) else {
            return self.room.reject(
                connection,
                request_id,
                RejectReason::Player(PlayerViolation::NotJoined),
            );
        };
        let Some(game) = self.game.as_mut() else {
            return self.room.reject(
                connection,
                request_id,
                RejectReason::Game(GameViolation::GameNotStarted),
            );
        };
        match game.declare(player, &cards) {
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

    pub(super) fn confirm_bid_pass(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
    ) -> Vec<Delivery> {
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
            .is_some_and(|game| matches!(game.phase(), ShengjiPhase::BiddingGrace))
        {
            return self.reject_game_error(connection, request_id, ShengjiGameError::WrongPhase);
        }
        let events = self.open_bot_declarations();
        self.flow.bid_pass_confirmed[usize::from(player.0)] = true;
        self.confirm_automatic_bid_passes();
        self.room.bump_revision();
        let mut deliveries = self.broadcast_events(events);
        if self
            .flow
            .bid_pass_confirmed
            .iter()
            .take(usize::from(self.rules.player_count()))
            .all(|confirmed| *confirmed)
        {
            deliveries.extend(self.finish_bidding(Some((connection, request_id))));
        } else {
            deliveries.extend(self.broadcast_game(Some((connection, request_id))));
        }
        deliveries
    }

    pub(super) fn reset_bid_pass_confirmations(&mut self) {
        self.flow.bid_pass_confirmed = [false; ShengjiRuleSet::MAX_PLAYERS as usize];
        self.confirm_automatic_bid_passes();
    }

    pub(super) fn confirm_automatic_bid_passes(&mut self) {
        for participant in &self.room.players {
            if usize::from(participant.id.0) >= usize::from(self.rules.player_count()) {
                continue;
            }
            if participant.is_bot
                || participant.auto_play
                || !participant.connected
                || participant.left
            {
                self.flow.bid_pass_confirmed[usize::from(participant.id.0)] = true;
            }
        }
    }

    pub(super) fn bury(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        cards: Vec<ShengjiCard>,
    ) -> Vec<Delivery> {
        self.apply_card_action(connection, request_id, |game, player| {
            game.bury(player, &cards)
        })
    }

    pub(super) fn choose_bottom_copy(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        cards: Option<Vec<ShengjiCard>>,
    ) -> Vec<Delivery> {
        self.apply_card_action(connection, request_id, |game, player| {
            game.choose_bottom_copy(player, cards.as_deref())
        })
    }

    pub(super) fn choose_five_trump_crossing(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        cards: Option<Vec<ShengjiCard>>,
    ) -> Vec<Delivery> {
        self.apply_card_action(connection, request_id, |game, player| {
            game.choose_five_trump_crossing(player, cards.as_deref())
        })
    }

    pub(super) fn return_five_trump_crossing(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        cards: Vec<ShengjiCard>,
    ) -> Vec<Delivery> {
        self.apply_card_action(connection, request_id, |game, player| {
            game.return_five_trump_crossing(player, &cards)
        })
    }

    pub(super) fn play_cards(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        cards: Vec<ShengjiCard>,
    ) -> Vec<Delivery> {
        self.apply_card_action(connection, request_id, |game, player| {
            game.play_cards(player, &cards)
        })
    }

    fn apply_card_action(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        action: impl FnOnce(
            &mut ShengjiGameState,
            PlayerId,
        ) -> Result<ShengjiActionOutcome, ShengjiGameError>,
    ) -> Vec<Delivery> {
        if self.presentation.trick.is_some() || self.presentation.throw_failure.is_some() {
            return self.room.reject(
                connection,
                request_id,
                RejectReason::Game(GameViolation::Shengji(ShengjiViolation::WrongPhase)),
            );
        }
        let Some(player) = self.room.player_id(connection) else {
            return self.room.reject(
                connection,
                request_id,
                RejectReason::Player(PlayerViolation::NotJoined),
            );
        };
        let Some(game) = self.game.as_mut() else {
            return self.room.reject(
                connection,
                request_id,
                RejectReason::Game(GameViolation::GameNotStarted),
            );
        };
        let outcome = match action(game, player) {
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
}
