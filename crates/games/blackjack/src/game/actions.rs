use super::{
    BlackjackAction, BlackjackAvailableActions, BlackjackGameError, BlackjackGameState,
    BlackjackHandState, BlackjackHandStatus, BlackjackPhase,
};
use crate::{BlackjackRank, BlackjackRuleSet};
use leocard_game_common::PlayerId;

impl BlackjackGameState {
    pub fn available_actions(&self, player: PlayerId) -> BlackjackAvailableActions {
        if self.current != Some(player) {
            return BlackjackAvailableActions::default();
        }
        let state = &self.players[player.index()];
        if self.phase == BlackjackPhase::Betting {
            return BlackjackAvailableActions {
                minimum_bet: self.rules.minimum_bet,
                maximum_bet: state.stack.min(self.rules.initial_chips) / 2 * 2,
                ..Default::default()
            };
        }
        if self.phase != BlackjackPhase::Playing {
            return BlackjackAvailableActions::default();
        }
        let hand = &state.hands[self.hand];
        let two_cards = hand.cards.len() == 2;
        BlackjackAvailableActions {
            hit: true,
            stand: true,
            double_down: two_cards && state.stack >= hand.wager,
            split: two_cards
                && state.stack >= hand.wager
                && state.hands.len() < BlackjackRuleSet::MAX_HANDS
                && hand.cards[0].rank.points() == hand.cards[1].rank.points(),
            ..Default::default()
        }
    }

    pub fn act(
        &mut self,
        player: PlayerId,
        action: BlackjackAction,
    ) -> Result<(), BlackjackGameError> {
        if self.current != Some(player) {
            return Err(BlackjackGameError::NotPlayersTurn);
        }
        let available = self.available_actions(player);
        match action {
            BlackjackAction::Bet { amount } => {
                if self.phase != BlackjackPhase::Betting {
                    return Err(BlackjackGameError::WrongPhase);
                }
                if amount < available.minimum_bet
                    || amount > available.maximum_bet
                    || !amount.is_multiple_of(2)
                {
                    return Err(BlackjackGameError::InvalidBet);
                }
                self.bet(player, amount);
                return Ok(());
            }
            BlackjackAction::Hit if available.hit => self.hit(player, false),
            BlackjackAction::Stand if available.stand => {
                self.players[player.index()].hands[self.hand].status =
                    BlackjackHandStatus::Standing;
            }
            BlackjackAction::DoubleDown => {
                if !available.double_down {
                    return Err(BlackjackGameError::CannotDouble);
                }
                let wager = self.players[player.index()].hands[self.hand].wager;
                self.players[player.index()].stack -= wager;
                self.players[player.index()].hands[self.hand].wager *= 2;
                self.players[player.index()].hands[self.hand].doubled = true;
                self.players[player.index()].statistics.doubles += 1;
                self.hit(player, true);
            }
            BlackjackAction::Split => {
                if !available.split {
                    return Err(BlackjackGameError::CannotSplit);
                }
                self.split(player);
            }
            _ => return Err(BlackjackGameError::WrongPhase),
        }
        self.advance_play();
        Ok(())
    }

    fn hit(&mut self, player: PlayerId, force_stand: bool) {
        let card = self.draw();
        let hand = &mut self.players[player.index()].hands[self.hand];
        hand.cards.push(card);
        hand.status = match hand.value().total {
            22.. => BlackjackHandStatus::Bust,
            21 => BlackjackHandStatus::Standing,
            _ if force_stand => BlackjackHandStatus::Standing,
            _ => BlackjackHandStatus::Playing,
        };
    }

    fn split(&mut self, player: PlayerId) {
        let left_card = self.draw();
        let right_card = self.draw();
        let state = &mut self.players[player.index()];
        let hand = &mut state.hands[self.hand];
        let card = hand.cards.pop().expect("split requires two cards");
        let aces = card.rank == BlackjackRank::Ace;
        hand.cards.push(left_card);
        hand.split = true;
        if aces || hand.value().total == 21 {
            hand.status = BlackjackHandStatus::Standing;
        }
        let mut right = BlackjackHandState {
            cards: vec![card, right_card],
            wager: hand.wager,
            status: BlackjackHandStatus::Playing,
            split: true,
            doubled: false,
            result: None,
            net_chips: 0,
        };
        if aces || right.value().total == 21 {
            right.status = BlackjackHandStatus::Standing;
        }
        state.stack -= right.wager;
        state.hands.insert(self.hand + 1, right);
        state.statistics.splits += 1;
    }
}
