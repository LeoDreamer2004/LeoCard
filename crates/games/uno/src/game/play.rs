use super::*;

impl GameState {
    pub fn can_play(&self, player: UnoPlayerId, card: UnoCard) -> bool {
        self.ensure_turn(player).is_ok()
            && self.pending_swap.is_none()
            && self.current_color.is_some()
            && self.players[player.0].hand.contains(&card)
            && self.card_allowed_in_current_state(card)
            && self.drawn_card.is_none_or(|drawn_card| drawn_card == card)
    }

    pub fn play_card(
        &mut self,
        player: UnoPlayerId,
        card: UnoCard,
        chosen_color: Option<UnoColor>,
    ) -> Result<ActionOutcome, GameError> {
        self.ensure_turn(player)?;
        self.ensure_swap_resolved()?;
        if self.current_color.is_none() {
            return Err(GameError::InitialColorChoiceRequired);
        }
        if !self.players[player.0].hand.contains(&card) {
            return Err(GameError::CardNotInHand(card));
        }
        let deferred_color = matches!(
            card.face(),
            UnoFace::WildForceTrade | UnoFace::WildPassHands | UnoFace::WildColorRoulette
        );
        if card.face().is_wild() {
            if deferred_color && chosen_color.is_some() {
                return Err(GameError::UnexpectedColor);
            }
            if !deferred_color && chosen_color.is_none() {
                return Err(GameError::ColorRequired);
            }
        } else if chosen_color.is_some() {
            return Err(GameError::UnexpectedColor);
        }
        if chosen_color.is_some_and(|color| !self.color_allowed(color)) {
            return Err(GameError::UnexpectedColor);
        }
        if let Some(drawn_card) = self.drawn_card
            && card != drawn_card
        {
            return Err(GameError::MustPlayDrawnCard(drawn_card));
        }
        if self.skip_turns[player.0] > 0 {
            return Err(GameError::MustResolveSkip);
        }
        if self.pending_skip > 0 {
            if !self.skip_stack_allowed(card) {
                return Err(GameError::MustResolveSkip);
            }
        } else if self.pending_draw_active() {
            if !self.stack_allowed(card) {
                return Err(GameError::CannotStack(card));
            }
        } else if !self.matches_top(card) {
            return Err(GameError::CardDoesNotMatch);
        }

        let previous_color = self.current_color.expect("checked above");
        let wild_draw_was_legal = if (self.rules.is_classic()
            && card.face() == UnoFace::WildDrawFour)
            || (self.rules.is_flip()
                && matches!(card.face(), UnoFace::WildDrawTwo | UnoFace::WildDrawColor))
        {
            match self.pending_kind {
                // A +4 stacked on a +2 can only be answered by the matching
                // +2 under the configured stacking rule. Other cards of the
                // same color are not playable while the draw penalty is open.
                Some(UnoPendingDrawKind::DrawTwo) => {
                    !self.players[player.0].hand.iter().any(|other| {
                        *other != card
                            && (other.face() == UnoFace::ReverseDrawTwo
                                || (other.face() == UnoFace::DrawTwo
                                    && other.color() == Some(previous_color)))
                    })
                }
                _ => !self.players[player.0]
                    .hand
                    .iter()
                    .any(|other| *other != card && other.color() == Some(previous_color)),
            }
        } else {
            true
        };
        let declared_uno = self.uno_declared[player.0];
        self.uno_exposed[player.0] = false;
        self.uno_declared[player.0] = false;
        self.drawn_card = None;
        self.jump_in_open = false;
        remove_card(&mut self.players[player.0].hand, card);
        self.discard_pile.push(card);
        self.current_color = if deferred_color {
            None
        } else {
            chosen_color.or(card.color())
        };

        if self.players[player.0].hand.is_empty() {
            self.pending_finisher.get_or_insert(player);
        }

        let mut effect = None;
        let next_player = match card.face() {
            UnoFace::DrawOne => {
                self.pending_draw = self.pending_draw.saturating_add(1);
                self.pending_kind = Some(UnoPendingDrawKind::FlipDrawOne);
                self.pending_draw_source = Some(player);
                self.challenge = None;
                self.next_player(player)
            }
            UnoFace::DrawTwo => {
                self.pending_draw += 2;
                self.pending_kind = Some(if self.rules.is_no_mercy() {
                    UnoPendingDrawKind::NoMercy(2)
                } else {
                    UnoPendingDrawKind::DrawTwo
                });
                self.pending_draw_source = Some(player);
                self.challenge = None;
                self.next_player(player)
            }
            UnoFace::DrawFour => {
                self.pending_draw += 4;
                self.pending_kind = Some(UnoPendingDrawKind::NoMercy(4));
                self.pending_draw_source = Some(player);
                self.challenge = None;
                self.next_player(player)
            }
            UnoFace::DrawFive => {
                self.pending_draw = self.pending_draw.saturating_add(5);
                self.pending_kind = Some(UnoPendingDrawKind::FlipDrawFive);
                self.pending_draw_source = Some(player);
                self.challenge = None;
                self.next_player(player)
            }
            UnoFace::WildDrawTwo => {
                self.pending_draw = self.pending_draw.saturating_add(2);
                self.pending_kind = Some(UnoPendingDrawKind::FlipWildDrawTwo);
                self.pending_draw_source = Some(player);
                if self.challenge.is_none() {
                    self.challenge = Some(ChallengeState {
                        offender: player,
                        was_legal: wild_draw_was_legal,
                    });
                }
                self.next_player(player)
            }
            UnoFace::WildDrawFour => {
                self.pending_draw += 4;
                self.pending_kind = Some(UnoPendingDrawKind::WildDrawFour);
                self.pending_draw_source = Some(player);
                if self.challenge.is_none() {
                    self.challenge = Some(ChallengeState {
                        offender: player,
                        was_legal: wild_draw_was_legal,
                    });
                }
                self.next_player(player)
            }
            UnoFace::WildDrawColor => {
                self.pending_draw = self.pending_draw.saturating_add(1);
                self.pending_kind = Some(UnoPendingDrawKind::FlipWildDrawColor);
                self.pending_draw_source = Some(player);
                self.pending_draw_colors
                    .push(chosen_color.expect("wild draw color requires a color"));
                if self.challenge.is_none() {
                    self.challenge = Some(ChallengeState {
                        offender: player,
                        was_legal: wild_draw_was_legal,
                    });
                }
                self.next_player(player)
            }
            UnoFace::Reverse if self.players.len() == 2 => player,
            UnoFace::Reverse => {
                self.direction = self.direction.reversed();
                self.next_player(player)
            }
            UnoFace::Skip => {
                self.pending_skip = if self.action_stacking_enabled() {
                    self.pending_skip_everyone = false;
                    self.pending_skip_source = Some(player);
                    self.pending_skip.saturating_add(1)
                } else {
                    let target = self.next_player(player);
                    self.skip_turns[target.0] = self.skip_turns[target.0].max(1);
                    0
                };
                self.next_player(player)
            }
            UnoFace::SkipEveryone if self.rules.is_flip() && self.action_stacking_enabled() => {
                self.pending_skip = self.pending_skip.saturating_add(1);
                self.pending_skip_everyone = true;
                self.pending_skip_source = Some(player);
                self.next_player(player)
            }
            UnoFace::SkipEveryone => player,
            UnoFace::Flip => {
                let side = self.flip_everything();
                effect = Some(PlayedEffect::Flipped { side });
                self.next_player(player)
            }
            UnoFace::DiscardAll => {
                let color = card.color().expect("discard-all is colored");
                let mut discarded = Vec::new();
                self.players[player.0].hand.retain(|candidate| {
                    if candidate.color() == Some(color) {
                        discarded.push(*candidate);
                        false
                    } else {
                        true
                    }
                });
                if !discarded.is_empty() {
                    let top = self.discard_pile.pop().expect("played card is on top");
                    self.discard_pile.extend(discarded.iter().copied());
                    self.discard_pile.push(top);
                    effect = Some(PlayedEffect::CardsDiscarded { cards: discarded });
                }
                if self.players[player.0].hand.is_empty() {
                    self.pending_finisher.get_or_insert(player);
                }
                self.next_player(player)
            }
            UnoFace::ReverseDrawTwo => {
                self.direction = self.direction.reversed();
                self.pending_draw += 2;
                self.pending_kind = Some(UnoPendingDrawKind::DrawTwo);
                self.pending_draw_source = Some(player);
                self.challenge = None;
                self.next_player(player)
            }
            UnoFace::ReverseSkip => {
                self.direction = self.direction.reversed();
                self.pending_skip = if self.rules.action_stacking {
                    self.pending_skip_everyone = false;
                    self.pending_skip_source = Some(player);
                    self.pending_skip.saturating_add(1)
                } else {
                    let target = self.next_player(player);
                    self.skip_turns[target.0] = self.skip_turns[target.0].max(1);
                    0
                };
                self.next_player(player)
            }
            UnoFace::StackOne => {
                self.pending_draw = self.pending_draw.saturating_add(1);
                self.pending_kind = Some(UnoPendingDrawKind::Stack);
                self.pending_draw_source = Some(player);
                self.next_player(player)
            }
            UnoFace::StackTwo => {
                self.pending_draw = self.pending_draw.saturating_add(2);
                self.pending_kind = Some(UnoPendingDrawKind::Stack);
                self.pending_draw_source = Some(player);
                self.next_player(player)
            }
            UnoFace::SwapOne => {
                self.pending_swap = Some(PendingSwapState::SwapOneTarget {
                    player,
                    declared_uno,
                });
                player
            }
            UnoFace::RefreshHand => {
                let count = self.refresh_hand(player)?;
                effect = Some(PlayedEffect::HandRefreshed { count });
                self.update_uno_after_play(player, declared_uno);
                self.next_player(player)
            }
            UnoFace::WildForceTrade => {
                self.pending_swap = Some(PendingSwapState::ForceTrade { player });
                player
            }
            UnoFace::WildPassHands => {
                self.pass_hands();
                self.uno_exposed.fill(false);
                self.uno_declared.fill(false);
                self.pending_swap = Some(PendingSwapState::ChooseColor { player });
                effect = Some(PlayedEffect::HandsPassed {
                    direction: self.direction,
                });
                player
            }
            UnoFace::WildPowerReverse => {
                self.direction = self.direction.reversed();
                player
            }
            UnoFace::WildNoU if self.pending_draw_active() => {
                self.direction = self.direction.reversed();
                let target = self
                    .pending_draw_source
                    .expect("a draw penalty records its latest source");
                let cards = self.draw_cards_for(target, self.pending_draw)?;
                self.clear_pending_draw();
                effect = Some(PlayedEffect::DrawReflected {
                    player: target,
                    cards,
                });
                self.next_player(target)
            }
            UnoFace::WildNoU => {
                self.direction = self.direction.reversed();
                self.next_player(player)
            }
            UnoFace::WildStackThree => {
                self.pending_draw = self.pending_draw.saturating_add(3);
                self.pending_kind = Some(UnoPendingDrawKind::Stack);
                self.pending_draw_source = Some(player);
                self.next_player(player)
            }
            UnoFace::WildStackNumber => {
                let (cards, value) = self.reveal_stack_number()?;
                self.pending_draw = self.pending_draw.saturating_add(u16::from(value));
                self.pending_kind = Some(UnoPendingDrawKind::Stack);
                self.pending_draw_source = Some(player);
                effect = Some(PlayedEffect::StackNumberRevealed { cards, value });
                self.next_player(player)
            }
            UnoFace::WildReverseDrawFour => {
                self.direction = self.direction.reversed();
                self.pending_draw = self.pending_draw.saturating_add(4);
                self.pending_kind = Some(UnoPendingDrawKind::NoMercy(4));
                self.pending_draw_source = Some(player);
                self.challenge = None;
                self.next_player(player)
            }
            UnoFace::WildDrawSix => {
                self.pending_draw = self.pending_draw.saturating_add(6);
                self.pending_kind = Some(UnoPendingDrawKind::NoMercy(6));
                self.pending_draw_source = Some(player);
                self.challenge = None;
                self.next_player(player)
            }
            UnoFace::WildDrawTen => {
                self.pending_draw = self.pending_draw.saturating_add(10);
                self.pending_kind = Some(UnoPendingDrawKind::NoMercy(10));
                self.pending_draw_source = Some(player);
                self.challenge = None;
                self.next_player(player)
            }
            UnoFace::WildColorRoulette => {
                let target = self.next_player(player);
                self.pending_swap = Some(PendingSwapState::ColorRoulette { player: target });
                target
            }
            UnoFace::Number(0) if self.rules.is_no_mercy() && self.rules.no_mercy.zero_pass => {
                self.pass_hands();
                self.uno_exposed.fill(false);
                self.uno_declared.fill(false);
                effect = Some(PlayedEffect::HandsPassed {
                    direction: self.direction,
                });
                self.next_player(player)
            }
            UnoFace::Number(7) if self.rules.is_no_mercy() && self.rules.no_mercy.seven_swap => {
                self.pending_swap = Some(PendingSwapState::SevenSwap { player });
                player
            }
            UnoFace::Number(_) | UnoFace::Wild | UnoFace::DarkWild => self.next_player(player),
        };
        let no_mercy_hand_effect = self.rules.is_no_mercy()
            && (matches!(card.face(), UnoFace::Number(0)) && self.rules.no_mercy.zero_pass
                || matches!(card.face(), UnoFace::Number(7)) && self.rules.no_mercy.seven_swap);
        if !matches!(
            card.face(),
            UnoFace::SwapOne
                | UnoFace::RefreshHand
                | UnoFace::WildForceTrade
                | UnoFace::WildPassHands
        ) && !no_mercy_hand_effect
        {
            self.update_uno_after_play(player, declared_uno);
        }
        self.current_player = next_player;
        if !matches!(
            card.face(),
            UnoFace::SwapOne
                | UnoFace::RefreshHand
                | UnoFace::WildForceTrade
                | UnoFace::WildPassHands
                | UnoFace::WildColorRoulette
        ) && !no_mercy_hand_effect
        {
            self.jump_in_open = self.jump_in_enabled() && card.color().is_some();
        }
        if self.pending_swap.is_none()
            && !self.pending_draw_active()
            && self.pending_skip == 0
            && self.skip_turns[self.current_player.0] == 0
        {
            self.finish_pending_game();
        }
        Ok(ActionOutcome::Played {
            player,
            card,
            next_player,
            effect,
        })
    }
}
