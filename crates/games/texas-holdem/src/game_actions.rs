use super::*;

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
        if matches!(self.phase, Phase::Complete(_)) {
            return Err(GameError::HandAlreadyComplete);
        }
        let expected = self
            .current_player
            .expect("betting phase has a current player");
        if player != expected {
            return Err(GameError::NotPlayersTurn {
                expected,
                actual: player,
            });
        }
        if let Some(kind) = self.pending_blind {
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
                return Ok(ActionOutcome::BlindPosted {
                    player,
                    kind,
                    amount,
                    next_player: self.current_player,
                });
            }

            self.pending_blind = None;
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
                let Phase::Complete(result) = &self.phase else {
                    unreachable!("无人可行动时应自动摊牌")
                };
                return Ok(ActionOutcome::HandComplete(result.clone()));
            }
            return Ok(ActionOutcome::BlindPosted {
                player,
                kind,
                amount,
                next_player: self.current_player,
            });
        }
        if action == TexasHoldemAction::PostBlind {
            return Err(GameError::NoBlindToPost);
        }
        if !self.can_act(player) {
            return Err(GameError::PlayerCannotAct(player));
        }

        let old_street = self.street();
        let old_community = self.community.len();
        let amount_to_call = self.amount_to_call(player)?;
        self.needs_action[player.0] = false;
        let mut full_raise = false;

        match action {
            TexasHoldemAction::PostBlind => unreachable!("盲注动作已在常规下注前处理"),
            TexasHoldemAction::Fold => {
                self.players[player.0].folded = true;
                self.raise_allowed[player.0] = false;
            }
            TexasHoldemAction::Check => {
                if amount_to_call > 0 {
                    return Err(GameError::CannotCheckWhileFacingBet { amount_to_call });
                }
                // Check 并没有回应任何下注。若后手玩家随后用不足最低下注额的
                // all-in 首次下注，当前玩家仍有权进行一次完整加注。
            }
            TexasHoldemAction::Call => {
                if amount_to_call == 0 {
                    return Err(GameError::NothingToCall);
                }
                let payment = amount_to_call.min(self.players[player.0].stack);
                self.commit(player, payment);
                self.raise_allowed[player.0] = false;
            }
            TexasHoldemAction::RaiseTo(target) => {
                if !self.raise_allowed[player.0] {
                    return Err(GameError::RaiseNotReopened);
                }
                if target <= self.current_bet {
                    return Err(GameError::RaiseMustExceedCurrentBet {
                        current_bet: self.current_bet,
                        target,
                    });
                }
                let maximum_target =
                    self.players[player.0].committed_street + self.players[player.0].stack;
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
                full_raise = true;
                // 主动加注已经用掉本次行动权；只有其他玩家后续的完整加注才能
                // 再次开放。否则，紧随其后的不足额 all-in 会错误地允许原加注者
                // 再次加注。
                self.raise_allowed[player.0] = false;
            }
            TexasHoldemAction::AllIn => {
                let target = self.players[player.0].committed_street + self.players[player.0].stack;
                if target > self.current_bet && !self.raise_allowed[player.0] {
                    return Err(GameError::RaiseNotReopened);
                }
                let payment = self.players[player.0].stack;
                self.commit(player, payment);
                if target > self.current_bet {
                    let raise_size = target - self.current_bet;
                    full_raise = raise_size >= self.minimum_raise;
                    if full_raise {
                        self.minimum_raise = raise_size;
                    }
                    self.current_bet = target;
                }
                self.raise_allowed[player.0] = false;
            }
        }

        if self.contenders().len() == 1 {
            let result = self.settle_uncontested();
            return Ok(ActionOutcome::HandComplete(result));
        }

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
