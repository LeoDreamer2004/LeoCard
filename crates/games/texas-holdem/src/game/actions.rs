use crate::TexasRuleSet;
use leocard_game_common::PlayerId;

use super::{
    TexasAction, TexasActionOutcome, TexasBettingRecord, TexasBlindKind, TexasGameError,
    TexasGameState, TexasPhase, TexasPlayerState, TexasStreet,
};

impl TexasGameState {
    pub fn act(
        &mut self,
        player: PlayerId,
        action: TexasAction,
    ) -> Result<TexasActionOutcome, TexasGameError> {
        let previous = self.clone();
        match self.act_inner(player, action) {
            Ok(outcome) => {
                self.record_action_statistics(&previous, player, action);
                self.betting_history.push(TexasBettingRecord {
                    player,
                    street: previous.street(),
                    action,
                    payment: self.players[player.index()]
                        .committed_total
                        .saturating_sub(previous.players[player.index()].committed_total),
                    pot_before: previous.pot(),
                });
                Ok(outcome)
            }
            Err(error) => {
                *self = previous;
                Err(error)
            }
        }
    }

    fn act_inner(
        &mut self,
        player: PlayerId,
        action: TexasAction,
    ) -> Result<TexasActionOutcome, TexasGameError> {
        self.validate_actor(player)?;
        if let Some(kind) = self.pending_blind {
            return self.post_blind(player, action, kind);
        }
        if action == TexasAction::PostBlind {
            return Err(TexasGameError::NoBlindToPost);
        }
        if !self.can_act(player) {
            return Err(TexasGameError::PlayerCannotAct(player));
        }

        let old_street = self.street();
        let old_community = self.community.len();
        let full_raise = self.apply_betting_action(player, action)?;

        if self.contenders().len() == 1 {
            let result = self.settle_uncontested();
            return Ok(TexasActionOutcome::HandComplete(result));
        }

        self.mark_follow_up_actions(player, full_raise);
        self.advance_after_action(player, action, old_street, old_community)
    }

    fn validate_actor(&self, player: PlayerId) -> Result<(), TexasGameError> {
        if matches!(self.phase, TexasPhase::Complete(_)) {
            return Err(TexasGameError::HandAlreadyComplete);
        }
        let expected = self
            .current_player
            .expect("betting phase has a current player");
        if player == expected {
            Ok(())
        } else {
            Err(TexasGameError::NotPlayersTurn {
                expected,
                actual: player,
            })
        }
    }

    fn post_blind(
        &mut self,
        player: PlayerId,
        action: TexasAction,
        kind: TexasBlindKind,
    ) -> Result<TexasActionOutcome, TexasGameError> {
        if action != TexasAction::PostBlind {
            return Err(TexasGameError::MustPostBlind);
        }
        let requested = match kind {
            TexasBlindKind::Small => TexasRuleSet::SMALL_BLIND,
            TexasBlindKind::Big => TexasRuleSet::BIG_BLIND,
        };
        let amount = requested.min(self.players[player.index()].stack);
        self.commit(player, amount);

        if kind == TexasBlindKind::Small {
            self.pending_blind = Some(TexasBlindKind::Big);
            self.current_player = Some(self.big_blind);
        } else {
            self.pending_blind = None;
            self.open_preflop_betting()?;
        }

        if let TexasPhase::Complete(result) = &self.phase {
            return Ok(TexasActionOutcome::HandComplete(result.clone()));
        }
        Ok(TexasActionOutcome::BlindPosted {
            player,
            kind,
            amount,
            next_player: self.current_player,
        })
    }

    fn open_preflop_betting(&mut self) -> Result<(), TexasGameError> {
        // 大盲是翻前的首次下注，之后最多再加注三次。
        self.full_bets = 1;
        self.current_bet = self
            .players
            .iter()
            .map(TexasPlayerState::committed_street)
            .max()
            .unwrap_or(0);
        self.needs_action.fill(false);
        self.raise_allowed.fill(false);
        for index in 0..self.players.len() {
            if self.can_act(PlayerId::from_index(index)) {
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
        player: PlayerId,
        action: TexasAction,
    ) -> Result<bool, TexasGameError> {
        let amount_to_call = self.amount_to_call(player)?;
        self.needs_action[player.index()] = false;
        match action {
            TexasAction::PostBlind => unreachable!("盲注动作已在常规下注前处理"),
            TexasAction::Fold => {
                self.players[player.index()].folded = true;
                self.raise_allowed[player.index()] = false;
                Ok(false)
            }
            TexasAction::Check => {
                if amount_to_call > 0 {
                    return Err(TexasGameError::CannotCheckWhileFacingBet { amount_to_call });
                }
                Ok(false)
            }
            TexasAction::Call => {
                if amount_to_call == 0 {
                    return Err(TexasGameError::NothingToCall);
                }
                let payment = amount_to_call.min(self.players[player.index()].stack);
                self.commit(player, payment);
                self.raise_allowed[player.index()] = false;
                Ok(false)
            }
            TexasAction::RaiseTo(target) => self.raise_to(player, target),
            TexasAction::AllIn => self.go_all_in(player),
        }
    }

    fn raise_to(&mut self, player: PlayerId, target: u32) -> Result<bool, TexasGameError> {
        self.validate_raise_rights(player)?;
        if target <= self.current_bet {
            return Err(TexasGameError::RaiseMustExceedCurrentBet {
                current_bet: self.current_bet,
                target,
            });
        }
        let maximum_target =
            self.players[player.index()].committed_street + self.players[player.index()].stack;
        if target > maximum_target {
            return Err(TexasGameError::RaiseExceedsStack {
                maximum_target,
                target,
            });
        }
        let minimum_target = self.current_bet + self.minimum_raise;
        if target < minimum_target {
            return Err(TexasGameError::RaiseBelowMinimum {
                minimum_target,
                target,
            });
        }
        self.validate_fixed_limit_target(target)?;
        if !self.raise_sizing(player)?.contains(target) {
            return Err(TexasGameError::RaiseInvalidIncrement {
                increment: self.minimum_raise,
                target,
            });
        }

        let payment = target - self.players[player.index()].committed_street;
        self.commit(player, payment);
        self.record_full_raise(target);
        self.current_bet = target;
        self.raise_allowed[player.index()] = false;
        Ok(true)
    }

    fn go_all_in(&mut self, player: PlayerId) -> Result<bool, TexasGameError> {
        let target =
            self.players[player.index()].committed_street + self.players[player.index()].stack;
        if target > self.current_bet {
            self.validate_raise_rights(player)?;
            self.validate_fixed_limit_target(target)?;
        }
        let payment = self.players[player.index()].stack;
        self.commit(player, payment);
        let mut full_raise = false;
        if target > self.current_bet {
            let raise_size = target - self.current_bet;
            full_raise = raise_size >= self.minimum_raise;
            if full_raise {
                self.record_full_raise(target);
            }
            self.current_bet = target;
        }
        self.raise_allowed[player.index()] = false;
        Ok(full_raise)
    }

    fn mark_follow_up_actions(&mut self, player: PlayerId, full_raise: bool) {
        if full_raise {
            for index in 0..self.players.len() {
                if index != player.index() && self.can_act(PlayerId::from_index(index)) {
                    self.needs_action[index] = true;
                    self.raise_allowed[index] = true;
                }
            }
        } else {
            for index in 0..self.players.len() {
                if index != player.index()
                    && self.can_act(PlayerId::from_index(index))
                    && self.players[index].committed_street < self.current_bet
                {
                    self.needs_action[index] = true;
                }
            }
        }
    }

    fn advance_after_action(
        &mut self,
        player: PlayerId,
        action: TexasAction,
        old_street: TexasStreet,
        old_community: usize,
    ) -> Result<TexasActionOutcome, TexasGameError> {
        if let Some(next) = self.next_needing_action(player) {
            self.current_player = Some(next);
            return Ok(TexasActionOutcome::Acted {
                player,
                action,
                next_player: next,
            });
        }

        self.finish_betting_round()?;
        if let TexasPhase::Complete(result) = &self.phase {
            return Ok(TexasActionOutcome::HandComplete(result.clone()));
        }
        let current_player = self
            .current_player
            .expect("new street has an acting player");
        Ok(TexasActionOutcome::StreetAdvanced {
            street: self.street(),
            current_player,
            community_cards: self.community.len() - old_community,
        })
        .inspect(|_| debug_assert_ne!(self.street(), old_street))
    }
}
