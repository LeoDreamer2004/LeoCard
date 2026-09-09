use super::{ShengjiSession, shuffled_deck, to_core_player};
use crate::lifecycle::HostedGameLifecycle;
use crate::{ConnectionId, Delivery};
use leocard_protocol::{
    GameViolation, PlayerViolation, RejectReason, RequestId, RoomViolation, ShengjiEvent,
    ShengjiProfileStats, ShengjiViolation,
};
use leocard_shengji::{
    ActionOutcome, GameError, GameState, Phase, ShengjiCard, ShengjiPlayerId, ShengjiRuleSet,
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
        if self.rules != rules {
            self.rules = rules;
            self.room.reset_ready_after_rules_change();
            self.shuffled_deck = Some(shuffled_deck(self.rules));
            self.room.bump_revision();
        }
        self.broadcast_lobby(Some((connection, request_id)))
    }

    pub(super) fn start_hand(&mut self, count_as_next_hand: bool) -> Result<(), GameError> {
        let deck = self
            .shuffled_deck
            .take()
            .unwrap_or_else(|| shuffled_deck(self.rules));
        let fixed_dealer = self.next_dealer;
        self.game = Some(GameState::new(
            self.rules,
            self.teams.clone(),
            fixed_dealer,
            fixed_dealer.unwrap_or(ShengjiPlayerId(0)),
            deck,
        )?);
        if count_as_next_hand || self.hand_number == 0 {
            self.hand_number = self.hand_number.saturating_add(1);
        } else {
            // 无人亮主重新发牌也需要让客户端重置逐张发牌动画。
            self.hand_number = self.hand_number.saturating_add(1);
        }
        self.flow.deal_elapsed = Duration::ZERO;
        self.flow.bidding_remaining = None;
        self.flow.bid_pass_confirmed = [false; ShengjiRuleSet::PLAYER_COUNT];
        self.flow.bottom_flip_reveal = None;
        self.flow.bottom_flip_remaining = None;
        self.flow.bottom_copy_remaining = None;
        self.flow.automatic_action = None;
        self.flow.redeal_remaining = None;
        self.presentation.throw_penalties = [0; ShengjiRuleSet::PLAYER_COUNT];
        self.presentation.throw_failure = None;
        self.presentation.trick = None;
        self.statistics
            .profiles
            .fill(ShengjiProfileStats::default());
        self.statistics.finished_settlement_id = None;
        self.statistics.finished_reference_changes = None;
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
            .is_some_and(|game| matches!(game.phase(), Phase::BiddingGrace))
        {
            return self.reject_game_error(connection, request_id, GameError::WrongPhase);
        }
        self.flow.bid_pass_confirmed[usize::from(player.0)] = true;
        self.confirm_automatic_bid_passes();
        self.room.bump_revision();
        if self
            .flow
            .bid_pass_confirmed
            .iter()
            .all(|confirmed| *confirmed)
        {
            self.finish_bidding(Some((connection, request_id)))
        } else {
            self.broadcast_game(Some((connection, request_id)))
        }
    }

    pub(super) fn reset_bid_pass_confirmations(&mut self) {
        self.flow.bid_pass_confirmed = [false; ShengjiRuleSet::PLAYER_COUNT];
        self.confirm_automatic_bid_passes();
    }

    pub(super) fn confirm_automatic_bid_passes(&mut self) {
        for participant in &self.room.players {
            if usize::from(participant.id.0) >= ShengjiRuleSet::PLAYER_COUNT {
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
        action: impl FnOnce(&mut GameState, ShengjiPlayerId) -> Result<ActionOutcome, GameError>,
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
}
