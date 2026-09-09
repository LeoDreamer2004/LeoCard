use super::{
    ActionOutcome, GameError, GameState, Phase, PlayerState, TexasHoldemAction,
    TexasHoldemBlindKind, TexasHoldemPlayerId, TexasHoldemRuleSet, TexasHoldemStreet,
};

impl GameState {
    pub fn act(
        &mut self,
        player: TexasHoldemPlayerId,
        action: TexasHoldemAction,
    ) -> Result<ActionOutcome, GameError> {
        let previous = self.clone();
        match self.act_inner(player, action) {
            Ok(outcome) => Ok(outcome),
            Err(error) => {
                *self = previous;
                Err(error)
            }
        }
    }

    fn act_inner(
        &mut self,
        player: TexasHoldemPlayerId,
        action: TexasHoldemAction,
    ) -> Result<ActionOutcome, GameError> {
        self.validate_actor(player)?;
        if let Some(kind) = self.pending_blind {
            return self.post_blind(player, action, kind);
        }
        if action == TexasHoldemAction::PostBlind {
            return Err(GameError::NoBlindToPost);
        }
        if !self.can_act(player) {
            return Err(GameError::PlayerCannotAct(player));
        }

        let old_street = self.street();
        let old_community = self.community.len();
        let full_raise = self.apply_betting_action(player, action)?;

        if self.contenders().len() == 1 {
            let result = self.settle_uncontested();
            return Ok(ActionOutcome::HandComplete(result));
        }

        self.mark_follow_up_actions(player, full_raise);
        self.advance_after_action(player, action, old_street, old_community)
    }

    fn validate_actor(&self, player: TexasHoldemPlayerId) -> Result<(), GameError> {
        if matches!(self.phase, Phase::Complete(_)) {
            return Err(GameError::HandAlreadyComplete);
        }
        let expected = self
            .current_player
            .expect("betting phase has a current player");
        if player == expected {
            Ok(())
        } else {
            Err(GameError::NotPlayersTurn {
                expected,
                actual: player,
            })
        }
    }

    fn post_blind(
        &mut self,
        player: TexasHoldemPlayerId,
        action: TexasHoldemAction,
        kind: TexasHoldemBlindKind,
    ) -> Result<ActionOutcome, GameError> {
        if action != TexasHoldemAction::PostBlind {
            return Err(GameError::MustPostBlind);
        }
        let requested = match kind {
            TexasHoldemBlindKind::Small => TexasHoldemRuleSet::SMALL_BLIND,
            TexasHoldemBlindKind::Big => TexasHoldemRuleSet::BIG_BLIND,
        };
        let amount = requested.min(self.players[player.0].stack);
        self.commit(player, amount);

        if kind == TexasHoldemBlindKind::Small {
            self.pending_blind = Some(TexasHoldemBlindKind::Big);
            self.current_player = Some(self.big_blind);
        } else {
            self.pending_blind = None;
            self.open_preflop_betting()?;
        }

        if let Phase::Complete(result) = &self.phase {
            return Ok(ActionOutcome::HandComplete(result.clone()));
        }
        Ok(ActionOutcome::BlindPosted {
            player,
            kind,
            amount,
            next_player: self.current_player,
        })
    }

    fn open_preflop_betting(&mut self) -> Result<(), GameError> {
        self.current_bet = self
            .players
            .iter()
            .map(PlayerState::committed_street)
            .max()
            .unwrap_or(0);
        self.needs_action.fill(false);
        self.raise_allowed.fill(false);
        for index in 0..self.players.len() {
            if self.can_act(TexasHoldemPlayerId(index)) {
                self.needs_action[index] = true;
                self.raise_allowed[index] = true;
            }
        }
        self.current_player = self.next_needing_action(self.big_blind);
        if self.current_player.is_none() {
            self.run_out_and_showdown()?;
        }
        Ok(())
    }

    fn apply_betting_action(
        &mut self,
        player: TexasHoldemPlayerId,
        action: TexasHoldemAction,
    ) -> Result<bool, GameError> {
        let amount_to_call = self.amount_to_call(player)?;
        self.needs_action[player.0] = false;
        match action {
            TexasHoldemAction::PostBlind => unreachable!("盲注动作已在常规下注前处理"),
            TexasHoldemAction::Fold => {
                self.players[player.0].folded = true;
                self.raise_allowed[player.0] = false;
                Ok(false)
            }
            TexasHoldemAction::Check => {
                if amount_to_call > 0 {
                    return Err(GameError::CannotCheckWhileFacingBet { amount_to_call });
                }
                Ok(false)
            }
            TexasHoldemAction::Call => {
                if amount_to_call == 0 {
                    return Err(GameError::NothingToCall);
                }
                let payment = amount_to_call.min(self.players[player.0].stack);
                self.commit(player, payment);
                self.raise_allowed[player.0] = false;
                Ok(false)
            }
            TexasHoldemAction::RaiseTo(target) => self.raise_to(player, target),
            TexasHoldemAction::AllIn => self.go_all_in(player),
        }
    }

    fn raise_to(&mut self, player: TexasHoldemPlayerId, target: u32) -> Result<bool, GameError> {
        if !self.raise_allowed[player.0] {
            return Err(GameError::RaiseNotReopened);
        }
        if target <= self.current_bet {
            return Err(GameError::RaiseMustExceedCurrentBet {
                current_bet: self.current_bet,
                target,
            });
        }
        let maximum_target = self.players[player.0].committed_street + self.players[player.0].stack;
        if target > maximum_target {
            return Err(GameError::RaiseExceedsStack {
                maximum_target,
                target,
            });
        }
        let minimum_target = self.current_bet + self.minimum_raise;
        if target < minimum_target {
            return Err(GameError::RaiseBelowMinimum {
                minimum_target,
                target,
            });
        }

        let payment = target - self.players[player.0].committed_street;
        self.commit(player, payment);
        self.minimum_raise = target - self.current_bet;
        self.current_bet = target;
        self.raise_allowed[player.0] = false;
        Ok(true)
    }

    fn go_all_in(&mut self, player: TexasHoldemPlayerId) -> Result<bool, GameError> {
        let target = self.players[player.0].committed_street + self.players[player.0].stack;
        if target > self.current_bet && !self.raise_allowed[player.0] {
            return Err(GameError::RaiseNotReopened);
        }
        let payment = self.players[player.0].stack;
        self.commit(player, payment);
        let mut full_raise = false;
        if target > self.current_bet {
            let raise_size = target - self.current_bet;
            full_raise = raise_size >= self.minimum_raise;
            if full_raise {
                self.minimum_raise = raise_size;
            }
            self.current_bet = target;
        }
        self.raise_allowed[player.0] = false;
        Ok(full_raise)
    }

    fn mark_follow_up_actions(&mut self, player: TexasHoldemPlayerId, full_raise: bool) {
        if full_raise {
            for index in 0..self.players.len() {
                if index != player.0 && self.can_act(TexasHoldemPlayerId(index)) {
                    self.needs_action[index] = true;
                    self.raise_allowed[index] = true;
                }
            }
        } else {
            for index in 0..self.players.len() {
                if index != player.0
                    && self.can_act(TexasHoldemPlayerId(index))
                    && self.players[index].committed_street < self.current_bet
                {
                    self.needs_action[index] = true;
                }
            }
        }
    }

    fn advance_after_action(
        &mut self,
        player: TexasHoldemPlayerId,
        action: TexasHoldemAction,
        old_street: TexasHoldemStreet,
        old_community: usize,
    ) -> Result<ActionOutcome, GameError> {
        if let Some(next) = self.next_needing_action(player) {
            self.current_player = Some(next);
            return Ok(ActionOutcome::Acted {
                player,
                action,
                next_player: next,
            });
        }

        self.finish_betting_round()?;
        if let Phase::Complete(result) = &self.phase {
            return Ok(ActionOutcome::HandComplete(result.clone()));
        }
        let current_player = self
            .current_player
            .expect("new street has an acting player");
        Ok(ActionOutcome::StreetAdvanced {
            street: self.street(),
            current_player,
            community_cards: self.community.len() - old_community,
        })
        .inspect(|_| debug_assert_ne!(self.street(), old_street))
    }
}
