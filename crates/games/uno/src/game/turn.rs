use super::*;

impl GameState {
    /// 非下家在窗口关闭前抢出与桌面牌颜色、牌面完全一致的另一张物理牌。
    pub fn jump_in(
        &mut self,
        player: UnoPlayerId,
        card: UnoCard,
    ) -> Result<ActionOutcome, GameError> {
        if self.jump_in_card(player) != Some(card) {
            return Err(GameError::CannotJumpIn);
        }
        self.current_player = player;
        self.play_card(player, card, None)
    }

    pub fn draw_card(&mut self, player: UnoPlayerId) -> Result<ActionOutcome, GameError> {
        self.ensure_turn(player)?;
        self.ensure_swap_resolved()?;
        self.ensure_skip_resolved(player)?;
        self.ensure_uno_followup_resolved(player)?;
        if self.current_color.is_none() {
            return Err(GameError::InitialColorChoiceRequired);
        }
        if self.pending_draw_active() {
            return Err(GameError::MustResolveDrawPenalty);
        }
        if self.drawn_card.is_some() {
            return Err(GameError::MustDrawBeforePassing);
        }
        let mut cards = self.draw_cards_for(player, 1)?;
        self.jump_in_open = false;
        loop {
            if self.check_mercy_elimination(player)
                || !self.rules.is_no_mercy()
                || !self.rules.no_mercy.draw_until_playable
                || cards
                    .last()
                    .is_some_and(|card| self.card_allowed_in_current_state(*card))
            {
                break;
            }
            cards.extend(self.draw_cards_for(player, 1)?);
        }
        let playable = if self.players[player.0].eliminated {
            None
        } else {
            cards
                .last()
                .copied()
                .filter(|card| self.card_allowed_in_current_state(*card))
        };
        let next_player = if let Some(card) = playable {
            self.drawn_card = Some(card);
            player
        } else {
            self.next_player(player)
        };
        self.current_player = next_player;
        Ok(ActionOutcome::DrewCards {
            player,
            cards,
            playable,
            next_player,
        })
    }

    pub fn pass_after_draw(&mut self, player: UnoPlayerId) -> Result<ActionOutcome, GameError> {
        self.ensure_turn(player)?;
        self.ensure_swap_resolved()?;
        self.ensure_skip_resolved(player)?;
        self.ensure_uno_followup_resolved(player)?;
        if self.rules.is_no_mercy()
            && self.rules.no_mercy.draw_until_playable
            && let Some(card) = self.drawn_card
        {
            return Err(GameError::MustPlayDrawnCard(card));
        }
        if self.drawn_card.is_none() {
            return Err(GameError::MustDrawBeforePassing);
        }
        self.drawn_card = None;
        self.jump_in_open = false;
        let next_player = self.next_player(player);
        self.current_player = next_player;
        Ok(ActionOutcome::PassedAfterDraw {
            player,
            next_player,
        })
    }

    pub fn accept_draw_penalty(&mut self, player: UnoPlayerId) -> Result<ActionOutcome, GameError> {
        self.ensure_turn(player)?;
        self.ensure_swap_resolved()?;
        self.ensure_uno_followup_resolved(player)?;
        if !self.pending_draw_active() {
            return Err(GameError::NoDrawPenalty);
        }
        let must_resolve_skip = self.pending_skip > 0 || self.skip_turns[player.0] > 0;
        let cards = self.draw_pending_penalty(player, 0)?;
        self.check_mercy_elimination(player);
        self.jump_in_open = false;
        self.clear_pending_draw();
        // 罚牌始终属于功能牌出牌者的直接下家。若该玩家同时仍被禁手，先由其
        // 收下罚牌并留在当前回合，随后再单独消耗禁手，不能把罚牌传给下下家。
        let next_player = if must_resolve_skip {
            player
        } else {
            self.next_player(player)
        };
        self.current_player = next_player;
        self.finish_pending_game();
        Ok(ActionOutcome::PenaltyDrawn {
            player,
            cards,
            next_player,
        })
    }

    pub fn challenge_draw_four(&mut self, player: UnoPlayerId) -> Result<ActionOutcome, GameError> {
        self.ensure_turn(player)?;
        self.ensure_swap_resolved()?;
        self.ensure_uno_followup_resolved(player)?;
        let challenge = self.challenge.ok_or(GameError::CannotChallenge)?;
        let must_resolve_skip = self.pending_skip > 0 || self.skip_turns[player.0] > 0;
        let (result, penalized, extra, next_player) = if challenge.was_legal {
            (
                UnoChallengeResult::Failed,
                player,
                2,
                if must_resolve_skip {
                    player
                } else {
                    self.next_player(player)
                },
            )
        } else {
            (
                UnoChallengeResult::Successful,
                challenge.offender,
                0,
                player,
            )
        };
        let cards = self.draw_pending_penalty(penalized, extra)?;
        self.check_mercy_elimination(penalized);
        self.jump_in_open = false;
        self.clear_pending_draw();
        self.current_player = next_player;
        self.finish_pending_game();
        Ok(ActionOutcome::ChallengeResolved {
            challenger: player,
            offender: challenge.offender,
            result,
            penalized,
            cards,
            next_player,
        })
    }

    pub fn call_uno(&mut self, player: UnoPlayerId) -> Result<ActionOutcome, GameError> {
        self.ensure_playing()?;
        self.ensure_player(player)?;
        if self.players[player.0].eliminated {
            return Err(GameError::PlayerEliminated(player));
        }
        if !self.rules.uno_callout() {
            return Err(GameError::UnoCalloutDisabled);
        }
        if self.uno_declared[player.0] {
            return Err(GameError::CannotCallUno(player));
        }
        let can_recover_after_play = self.can_recover_uno(player);
        let can_declare_before_play = self.can_declare_uno(player);
        if !can_declare_before_play && !can_recover_after_play {
            return Err(GameError::CannotCallUno(player));
        }

        if can_recover_after_play {
            self.uno_exposed[player.0] = false;
        } else {
            self.uno_declared[player.0] = true;
        }
        if player == self.current_player {
            self.jump_in_open = false;
        }
        Ok(ActionOutcome::UnoCalled { player })
    }

    pub fn can_call_uno(&self, player: UnoPlayerId) -> bool {
        matches!(self.phase, Phase::Playing)
            && self.rules.uno_callout()
            && self
                .players
                .get(player.0)
                .is_some_and(|state| !state.eliminated)
            && !self.uno_declared.get(player.0).copied().unwrap_or(false)
            && (self.can_declare_uno(player) || self.can_recover_uno(player))
    }

    fn can_declare_uno(&self, player: UnoPlayerId) -> bool {
        player == self.current_player
            && self.pending_swap.is_none()
            && self.players[player.0].hand.len() == 2
            && !self.pending_draw_active()
            && self.pending_skip == 0
            && self.skip_turns[player.0] == 0
            && self.players[player.0]
                .hand
                .iter()
                .copied()
                .any(|card| self.can_play(player, card))
    }

    fn can_recover_uno(&self, player: UnoPlayerId) -> bool {
        self.players[player.0].hand.len() == 1 && self.uno_exposed[player.0]
    }

    pub fn report_uno(
        &mut self,
        reporter: UnoPlayerId,
        target: UnoPlayerId,
    ) -> Result<ActionOutcome, GameError> {
        self.ensure_playing()?;
        self.ensure_player(reporter)?;
        self.ensure_player(target)?;
        if self.players[reporter.0].eliminated {
            return Err(GameError::PlayerEliminated(reporter));
        }
        if !self.rules.uno_callout() {
            return Err(GameError::UnoCalloutDisabled);
        }
        if reporter == target {
            return Err(GameError::CannotReportSelf);
        }
        if !self.uno_exposed[target.0] {
            return Err(GameError::PlayerNotReportable(target));
        }
        let cards = self.draw_cards_for(target, 2)?;
        self.check_mercy_elimination(target);
        if reporter == self.current_player {
            self.jump_in_open = false;
        }
        self.uno_exposed[target.0] = false;
        Ok(ActionOutcome::UnoReported {
            reporter,
            target,
            cards,
        })
    }

    pub fn resolve_skip(&mut self, player: UnoPlayerId) -> Result<ActionOutcome, GameError> {
        self.ensure_turn(player)?;
        self.ensure_swap_resolved()?;
        self.ensure_uno_followup_resolved(player)?;
        if self.pending_draw_active() {
            return Err(GameError::MustResolveDrawPenalty);
        }
        let pending = self.pending_skip;
        if self.pending_skip_everyone {
            let source = self
                .pending_skip_source
                .expect("stacked skip-everyone records its latest source");
            for state in &self.players {
                if !state.eliminated && state.id != source {
                    self.skip_turns[state.id.0] =
                        self.skip_turns[state.id.0].saturating_add(pending);
                }
            }
        }
        let existing = self.skip_turns[player.0];
        if pending == 0 && existing == 0 {
            return Err(GameError::NoSkipToResolve);
        }
        let total = if self.pending_skip_everyone {
            existing
        } else if self.action_stacking_enabled() {
            existing.saturating_add(pending)
        } else {
            existing.max(pending.min(1))
        };
        let cards = if self.skip_draw_penalty_enabled() {
            self.draw_cards_for(player, 1)?
        } else {
            Vec::new()
        };
        self.check_mercy_elimination(player);
        self.jump_in_open = false;
        self.pending_skip = 0;
        self.pending_skip_everyone = false;
        self.pending_skip_source = None;
        let remaining = total.saturating_sub(1);
        self.skip_turns[player.0] = remaining;
        self.drawn_card = None;
        let next_player = self.next_player(player);
        self.current_player = next_player;
        self.finish_pending_game();
        Ok(ActionOutcome::SkipResolved {
            player,
            cards,
            remaining,
            next_player,
        })
    }
}
