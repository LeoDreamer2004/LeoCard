use super::{GameError, GameState, UnoCard, UnoColor, UnoFace, UnoPendingDrawKind, UnoPlayerId};

pub(super) struct ValidatedPlay {
    pub(super) deferred_color: bool,
    pub(super) wild_draw_was_legal: bool,
    pub(super) declared_uno: bool,
}

impl GameState {
    pub fn can_play(&self, player: UnoPlayerId, card: UnoCard) -> bool {
        self.ensure_turn(player).is_ok()
            && self.pending_swap.is_none()
            && self.current_color.is_some()
            && self.players[player.0].hand.contains(&card)
            && self.card_allowed_in_current_state(card)
            && self.drawn_card.is_none_or(|drawn_card| drawn_card == card)
    }

    pub(super) fn validate_play(
        &self,
        player: UnoPlayerId,
        card: UnoCard,
        chosen_color: Option<UnoColor>,
    ) -> Result<ValidatedPlay, GameError> {
        self.ensure_turn(player)?;
        self.ensure_swap_resolved()?;
        let previous_color = self
            .current_color
            .ok_or(GameError::InitialColorChoiceRequired)?;
        if !self.players[player.0].hand.contains(&card) {
            return Err(GameError::CardNotInHand(card));
        }

        let deferred_color = matches!(
            card.face(),
            UnoFace::WildForceTrade | UnoFace::WildPassHands | UnoFace::WildColorRoulette
        );
        match (card.face().is_wild(), deferred_color, chosen_color) {
            (true, true, Some(_)) | (false, _, Some(_)) => {
                return Err(GameError::UnexpectedColor);
            }
            (true, false, None) => return Err(GameError::ColorRequired),
            _ => {}
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

        let challengeable_wild = self.rules.is_classic() && card.face() == UnoFace::WildDrawFour
            || self.rules.is_flip()
                && matches!(card.face(), UnoFace::WildDrawTwo | UnoFace::WildDrawColor);
        let wild_draw_was_legal = !challengeable_wild
            || match self.pending_kind {
                Some(UnoPendingDrawKind::DrawTwo) => {
                    !self.players[player.0].hand.iter().any(|other| {
                        *other != card
                            && (other.face() == UnoFace::ReverseDrawTwo
                                || other.face() == UnoFace::DrawTwo
                                    && other.color() == Some(previous_color))
                    })
                }
                _ => !self.players[player.0]
                    .hand
                    .iter()
                    .any(|other| *other != card && other.color() == Some(previous_color)),
            };

        Ok(ValidatedPlay {
            deferred_color,
            wild_draw_was_legal,
            declared_uno: self.uno_declared[player.0],
        })
    }
}
