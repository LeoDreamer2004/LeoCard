use super::{
    AvailableActions, BlackjackAction, GameError, GameState, HandState, HandStatus, Phase,
};
use crate::{BlackjackRank, BlackjackRuleSet};

impl GameState {
    pub fn available_actions(&self, player: usize) -> AvailableActions {
        if self.current != Some(player) {
            return AvailableActions::default();
        }
        let state = &self.players[player];
        if self.phase == Phase::Betting {
            return AvailableActions {
                minimum_bet: self.rules.minimum_bet,
                maximum_bet: state.stack.min(self.rules.initial_chips) / 2 * 2,
                ..Default::default()
            };
        }
        if self.phase != Phase::Playing {
            return AvailableActions::default();
        }
        let hand = &state.hands[self.hand];
        let two_cards = hand.cards.len() == 2;
        AvailableActions {
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

    pub fn act(&mut self, player: usize, action: BlackjackAction) -> Result<(), GameError> {
        if self.current != Some(player) {
            return Err(GameError::NotPlayersTurn);
        }
        let available = self.available_actions(player);
        match action {
            BlackjackAction::Bet { amount } => {
                if self.phase != Phase::Betting {
                    return Err(GameError::WrongPhase);
                }
                if amount < available.minimum_bet
                    || amount > available.maximum_bet
                    || !amount.is_multiple_of(2)
                {
                    return Err(GameError::InvalidBet);
                }
                self.bet(player, amount);
                return Ok(());
            }
            BlackjackAction::Hit if available.hit => self.hit(player, false),
            BlackjackAction::Stand if available.stand => {
                self.players[player].hands[self.hand].status = HandStatus::Standing;
            }
            BlackjackAction::DoubleDown => {
                if !available.double_down {
                    return Err(GameError::CannotDouble);
                }
                let wager = self.players[player].hands[self.hand].wager;
                self.players[player].stack -= wager;
                self.players[player].hands[self.hand].wager *= 2;
                self.players[player].hands[self.hand].doubled = true;
                self.players[player].statistics.doubles += 1;
                self.hit(player, true);
            }
            BlackjackAction::Split => {
                if !available.split {
                    return Err(GameError::CannotSplit);
                }
                self.split(player);
            }
            _ => return Err(GameError::WrongPhase),
        }
        self.advance_play();
        Ok(())
    }

    fn hit(&mut self, player: usize, force_stand: bool) {
        let card = self.draw();
        let hand = &mut self.players[player].hands[self.hand];
        hand.cards.push(card);
        hand.status = match hand.value().total {
            22.. => HandStatus::Bust,
            21 => HandStatus::Standing,
            _ if force_stand => HandStatus::Standing,
            _ => HandStatus::Playing,
        };
    }

    fn split(&mut self, player: usize) {
        let left_card = self.draw();
        let right_card = self.draw();
        let state = &mut self.players[player];
        let hand = &mut state.hands[self.hand];
        let card = hand.cards.pop().expect("split requires two cards");
        let aces = card.rank == BlackjackRank::Ace;
        hand.cards.push(left_card);
        hand.split = true;
        if aces || hand.value().total == 21 {
            hand.status = HandStatus::Standing;
        }
        let mut right = HandState {
            cards: vec![card, right_card],
            wager: hand.wager,
            status: HandStatus::Playing,
            split: true,
            doubled: false,
            result: None,
            net_chips: 0,
        };
        if aces || right.value().total == 21 {
            right.status = HandStatus::Standing;
        }
        state.stack -= right.wager;
        state.hands.insert(self.hand + 1, right);
        state.statistics.splits += 1;
    }
}
