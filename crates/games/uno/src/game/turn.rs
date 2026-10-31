use crate::UnoCard;
use leocard_game_common::PlayerId;

use super::{UnoActionOutcome, UnoChallengeResult, UnoGameError, UnoGameState, UnoPhase};

impl UnoGameState {
    /// 非下家在窗口关闭前抢出与桌面牌颜色、牌面完全一致的另一张物理牌。
    pub fn jump_in(
        &mut self,
        player: PlayerId,
        card: UnoCard,
    ) -> Result<UnoActionOutcome, UnoGameError> {
        if self.jump_in_card(player) != Some(card) {
            return Err(UnoGameError::CannotJumpIn);
        }
        self.current_player = player;
        self.play_card(player, card, None)
    }

    pub fn draw_card(&mut self, player: PlayerId) -> Result<UnoActionOutcome, UnoGameError> {
        self.ensure_turn(player)?;
        self.ensure_swap_resolved()?;
        self.ensure_skip_resolved(player)?;
        self.ensure_uno_followup_resolved(player)?;
        if self.current_color.is_none() {
            return Err(UnoGameError::InitialColorChoiceRequired);
        }
        if self.pending_draw_active() {
            return Err(UnoGameError::MustResolveDrawPenalty);
        }
        if self.drawn_card.is_some() {
            return Err(UnoGameError::MustDrawBeforePassing);
        }
        if self.sudden_death_free_play() {
            return Err(UnoGameError::DrawPileExhausted);
        }
        let mut cards = Vec::new();
        while let Some(card) = self.draw_cards_for(player, 1)?.pop() {
            cards.push(card);
            if self.check_mercy_elimination(player)
                || !self.rules.is_no_mercy()
                || !self.rules.no_mercy.draw_until_playable
                || self.card_allowed_in_current_state(card)
            {
                break;
            }
        }
        // With an exhausted sudden-death pile, drawing also means voluntarily declining.
        self.jump_in_open = false;
        let playable = if self.players[player.index()].eliminated {
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
        } else if self.sudden_death_active() && self.draw_pile.is_empty() {
            self.sudden_death_missed(player)
        } else {
            self.sudden_death_misses = 0;
            self.next_player(player)
        };
        self.current_player = next_player;
        Ok(UnoActionOutcome::DrewCards {
            player,
            cards,
            playable,
            next_player,
        })
    }

    pub fn pass_after_draw(&mut self, player: PlayerId) -> Result<UnoActionOutcome, UnoGameError> {
        self.ensure_turn(player)?;
        self.ensure_swap_resolved()?;
        self.ensure_skip_resolved(player)?;
        self.ensure_uno_followup_resolved(player)?;
        if self.rules.is_no_mercy()
            && self.rules.no_mercy.draw_until_playable
            && let Some(card) = self.drawn_card
        {
            return Err(UnoGameError::MustPlayDrawnCard(card));
        }
        if self.drawn_card.is_none() {
            return Err(UnoGameError::MustDrawBeforePassing);
        }
        self.drawn_card = None;
        self.jump_in_open = false;
        let next_player = self.next_player(player);
        self.current_player = next_player;
        Ok(UnoActionOutcome::PassedAfterDraw {
            player,
            next_player,
        })
    }

    pub fn accept_draw_penalty(
        &mut self,
        player: PlayerId,
    ) -> Result<UnoActionOutcome, UnoGameError> {
        self.ensure_turn(player)?;
        self.ensure_swap_resolved()?;
        self.ensure_uno_followup_resolved(player)?;
        if !self.pending_draw_active() {
            return Err(UnoGameError::NoDrawPenalty);
        }
        let must_resolve_skip = self.pending_skip > 0 || self.skip_turns[player.index()] > 0;
        let cards = self.draw_pending_penalty(player, 0)?;
        self.check_mercy_elimination(player);
        self.jump_in_open = false;
        self.clear_pending_draw();
        // 罚牌始终属于功能牌出牌者的直接下家。若该玩家同时仍被禁手，先由其
        // 收下罚牌并留在当前回合，随后再单独消耗禁手，不能把罚牌传给下下家。
        let next_player = if must_resolve_skip {
            player
        } else if self.sudden_death_active() && self.draw_pile.is_empty() {
            self.sudden_death_missed(player)
        } else {
            self.next_player(player)
        };
        self.current_player = next_player;
        self.finish_pending_game();
        Ok(UnoActionOutcome::PenaltyDrawn {
            player,
            cards,
            next_player,
        })
    }

    pub fn challenge_draw_four(
        &mut self,
        player: PlayerId,
    ) -> Result<UnoActionOutcome, UnoGameError> {
        self.ensure_turn(player)?;
        self.ensure_swap_resolved()?;
        self.ensure_uno_followup_resolved(player)?;
        let challenge = self.challenge.ok_or(UnoGameError::CannotChallenge)?;
        let must_resolve_skip = self.pending_skip > 0 || self.skip_turns[player.index()] > 0;
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
        Ok(UnoActionOutcome::ChallengeResolved {
            challenger: player,
            offender: challenge.offender,
            result,
            penalized,
            cards,
            next_player,
        })
    }

    pub fn call_uno(&mut self, player: PlayerId) -> Result<UnoActionOutcome, UnoGameError> {
        self.ensure_playing()?;
        self.ensure_player(player)?;
        if self.players[player.index()].eliminated {
            return Err(UnoGameError::PlayerEliminated(player));
        }
        if !self.rules.uno_callout() {
            return Err(UnoGameError::UnoCalloutDisabled);
        }
        if self.uno_declared[player.index()] {
            return Err(UnoGameError::CannotCallUno(player));
        }
        let can_recover_after_play = self.can_recover_uno(player);
        let can_declare_before_play = self.can_declare_uno(player);
        if !can_declare_before_play && !can_recover_after_play {
            return Err(UnoGameError::CannotCallUno(player));
        }

        if can_recover_after_play {
            self.uno_exposed[player.index()] = false;
        } else {
            self.uno_declared[player.index()] = true;
        }
        if player == self.current_player {
            self.jump_in_open = false;
        }
        Ok(UnoActionOutcome::UnoCalled { player })
    }

    pub fn can_call_uno(&self, player: PlayerId) -> bool {
        matches!(self.phase, UnoPhase::Playing)
            && self.rules.uno_callout()
            && self
                .players
                .get(player.index())
                .is_some_and(|state| !state.eliminated)
            && !self
                .uno_declared
                .get(player.index())
                .copied()
                .unwrap_or(false)
            && (self.can_declare_uno(player) || self.can_recover_uno(player))
    }

    fn can_declare_uno(&self, player: PlayerId) -> bool {
        player == self.current_player
            && self.pending_swap.is_none()
            && self.players[player.index()].hand.len() == 2
            && !self.pending_draw_active()
            && self.pending_skip == 0
            && self.skip_turns[player.index()] == 0
            && self.players[player.index()]
                .hand
                .iter()
                .copied()
                .any(|card| self.can_play(player, card))
    }

    fn can_recover_uno(&self, player: PlayerId) -> bool {
        self.players[player.index()].hand.len() == 1 && self.uno_exposed[player.index()]
    }

    pub fn report_uno(
        &mut self,
        reporter: PlayerId,
        target: PlayerId,
    ) -> Result<UnoActionOutcome, UnoGameError> {
        self.ensure_playing()?;
        self.ensure_player(reporter)?;
        self.ensure_player(target)?;
        if self.players[reporter.index()].eliminated {
            return Err(UnoGameError::PlayerEliminated(reporter));
        }
        if !self.rules.uno_callout() {
            return Err(UnoGameError::UnoCalloutDisabled);
        }
        if reporter == target {
            return Err(UnoGameError::CannotReportSelf);
        }
        if !self.uno_exposed[target.index()] {
            return Err(UnoGameError::PlayerNotReportable(target));
        }
        let cards = self.draw_cards_for(target, 2)?;
        self.check_mercy_elimination(target);
        if reporter == self.current_player {
            self.jump_in_open = false;
        }
        self.uno_exposed[target.index()] = false;
        Ok(UnoActionOutcome::UnoReported {
            reporter,
            target,
            cards,
        })
    }

    pub fn resolve_skip(&mut self, player: PlayerId) -> Result<UnoActionOutcome, UnoGameError> {
        self.ensure_turn(player)?;
        self.ensure_swap_resolved()?;
        self.ensure_uno_followup_resolved(player)?;
        if self.pending_draw_active() {
            return Err(UnoGameError::MustResolveDrawPenalty);
        }
        let pending = self.pending_skip;
        if self.pending_skip_everyone {
            let source = self
                .pending_skip_source
                .expect("stacked skip-everyone records its latest source");
            for state in &self.players {
                if !state.eliminated && state.id != source {
                    self.skip_turns[state.id.index()] =
                        self.skip_turns[state.id.index()].saturating_add(pending);
                    if !self.skip_effect_targets.contains(&state.id) {
                        self.skip_effect_targets.push(state.id);
                    }
                }
            }
        }
        let existing = self.skip_turns[player.index()];
        if pending == 0 && existing == 0 {
            return Err(UnoGameError::NoSkipToResolve);
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
        self.skip_turns[player.index()] = remaining;
        self.skip_effect_targets.retain(|target| *target != player);
        self.drawn_card = None;
        let next_player = if self.sudden_death_active() && self.draw_pile.is_empty() {
            self.sudden_death_missed(player)
        } else {
            self.next_player(player)
        };
        self.current_player = next_player;
        self.finish_pending_game();
        Ok(UnoActionOutcome::SkipResolved {
            player,
            cards,
            remaining,
            next_player,
        })
    }
}
