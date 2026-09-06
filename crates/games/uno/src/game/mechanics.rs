use super::*;

impl GameState {
    pub(super) fn apply_starting_card(&mut self) -> Result<(), GameError> {
        let card = *self.discard_pile.last().expect("starting card exists");
        match card.face() {
            UnoFace::DrawOne => {
                let player = self.current_player;
                self.draw_cards_for(player, 1)?;
                self.current_player = self.next_player(player);
            }
            UnoFace::DrawTwo => {
                let player = self.current_player;
                self.draw_cards_for(player, 2)?;
                self.current_player = self.next_player(player);
            }
            UnoFace::Reverse => {
                self.direction = UnoDirection::CounterClockwise;
                self.current_player = self.next_player(self.current_player);
            }
            UnoFace::Skip => {
                if self.action_stacking_enabled() {
                    self.pending_skip = 1;
                } else {
                    self.skip_turns[self.current_player.0] = 1;
                }
            }
            UnoFace::ReverseDrawTwo => {
                self.direction = self.direction.reversed();
                let player = self.current_player;
                self.draw_cards_for(player, 2)?;
                self.current_player = self.next_player(player);
            }
            UnoFace::ReverseSkip => {
                self.direction = self.direction.reversed();
                if self.rules.action_stacking {
                    self.pending_skip = 1;
                } else {
                    self.skip_turns[self.current_player.0] = 1;
                }
            }
            UnoFace::Flip => {
                self.flip_everything();
            }
            UnoFace::StackOne => {
                let player = self.current_player;
                self.draw_cards_for(player, 1)?;
                self.current_player = self.next_player(player);
            }
            UnoFace::StackTwo => {
                let player = self.current_player;
                self.draw_cards_for(player, 2)?;
                self.current_player = self.next_player(player);
            }
            UnoFace::Wild
            | UnoFace::DarkWild
            | UnoFace::WildForceTrade
            | UnoFace::WildPassHands
            | UnoFace::WildPowerReverse
            | UnoFace::WildNoU
            | UnoFace::WildStackThree
            | UnoFace::WildStackNumber
            | UnoFace::WildReverseDrawFour
            | UnoFace::WildDrawSix
            | UnoFace::WildDrawTen
            | UnoFace::WildColorRoulette => self.current_color = None,
            UnoFace::Number(_)
            | UnoFace::DrawFour
            | UnoFace::SkipEveryone
            | UnoFace::DiscardAll
            | UnoFace::SwapOne
            | UnoFace::RefreshHand => {}
            UnoFace::DrawFive | UnoFace::WildDrawColor => {}
            UnoFace::WildDrawFour | UnoFace::WildDrawTwo => {
                unreachable!("initial wild draw card was rotated away")
            }
        }
        Ok(())
    }

    pub(super) fn ensure_playing(&self) -> Result<(), GameError> {
        if matches!(self.phase, Phase::Finished(_)) {
            Err(GameError::GameAlreadyFinished)
        } else {
            Ok(())
        }
    }

    pub(super) fn ensure_player(&self, player: UnoPlayerId) -> Result<(), GameError> {
        if player.0 < self.players.len() {
            Ok(())
        } else {
            Err(GameError::InvalidPlayer(player))
        }
    }

    pub(super) fn ensure_turn(&self, player: UnoPlayerId) -> Result<(), GameError> {
        self.ensure_playing()?;
        self.ensure_player(player)?;
        if player == self.current_player && !self.players[player.0].eliminated {
            Ok(())
        } else {
            Err(GameError::NotPlayersTurn {
                expected: self.current_player,
                actual: player,
            })
        }
    }

    pub(super) fn ensure_skip_resolved(&self, player: UnoPlayerId) -> Result<(), GameError> {
        if self.pending_skip > 0 || self.skip_turns[player.0] > 0 {
            Err(GameError::MustResolveSkip)
        } else {
            Ok(())
        }
    }

    pub(super) fn ensure_swap_resolved(&self) -> Result<(), GameError> {
        if self.pending_swap.is_some() {
            Err(GameError::MustResolveSwapEffect)
        } else {
            Ok(())
        }
    }

    pub(super) fn ensure_uno_followup_resolved(
        &self,
        player: UnoPlayerId,
    ) -> Result<(), GameError> {
        if self.uno_declared[player.0] {
            Err(GameError::MustPlayAfterUno)
        } else {
            Ok(())
        }
    }

    pub(super) const fn color_allowed(&self, color: UnoColor) -> bool {
        match self.flip_side {
            Some(UnoFlipSide::Dark) => color.is_dark(),
            Some(UnoFlipSide::Light) | None => color.is_light(),
        }
    }

    pub(super) const fn action_stacking_enabled(&self) -> bool {
        if self.rules.is_flip() {
            self.rules.flip.action_stacking
        } else {
            self.rules.is_classic() && self.rules.action_stacking
        }
    }

    pub(super) const fn skip_draw_penalty_enabled(&self) -> bool {
        if self.rules.is_flip() {
            self.rules.flip.skip_draw_penalty
        } else {
            self.rules.is_classic() && self.rules.skip_draw_penalty
        }
    }

    pub(super) const fn jump_in_enabled(&self) -> bool {
        if self.rules.is_flip() {
            self.rules.flip.jump_in
        } else {
            self.rules.is_classic() && self.rules.jump_in
        }
    }

    pub(super) fn skip_stack_allowed(&self, card: UnoCard) -> bool {
        if !self.action_stacking_enabled() {
            return false;
        }
        if self.rules.is_flip() {
            return self
                .discard_pile
                .last()
                .is_some_and(|top| top.face() == card.face())
                && matches!(card.face(), UnoFace::Skip | UnoFace::SkipEveryone);
        }
        matches!(card.face(), UnoFace::Skip | UnoFace::ReverseSkip)
    }

    pub(super) fn flip_everything(&mut self) -> UnoFlipSide {
        for player in &mut self.players {
            for card in &mut player.hand {
                *card = card.flipped();
            }
            player.hand.sort_by(UnoCard::display_cmp);
        }
        for card in &mut self.draw_pile {
            *card = card.flipped();
        }
        self.draw_pile.make_contiguous().reverse();
        for card in &mut self.discard_pile {
            *card = card.flipped();
        }
        self.discard_pile.reverse();
        for card in &mut self.set_aside_cards {
            *card = card.flipped();
        }
        self.drawn_card = self.drawn_card.map(UnoCard::flipped);
        let side = match self.flip_side.unwrap_or(UnoFlipSide::Light) {
            UnoFlipSide::Light => UnoFlipSide::Dark,
            UnoFlipSide::Dark => UnoFlipSide::Light,
        };
        self.flip_side = Some(side);
        self.current_color = self.top_card().color();
        side
    }

    pub(super) fn matches_top(&self, card: UnoCard) -> bool {
        card.face().is_wild()
            || card.color() == self.current_color
            || self
                .discard_pile
                .last()
                .is_some_and(|top| faces_match(top.face(), card.face()))
    }

    pub(super) fn stack_allowed(&self, card: UnoCard) -> bool {
        if self.rules.is_no_mercy() {
            return match self.pending_kind {
                Some(UnoPendingDrawKind::NoMercy(minimum)) => card
                    .face()
                    .draw_value()
                    .is_some_and(|value| value >= minimum),
                _ => false,
            };
        }
        if self.rules.is_flip() {
            if !self.rules.flip.action_stacking {
                return false;
            }
            return matches!(
                (self.pending_kind, card.face()),
                (
                    Some(UnoPendingDrawKind::FlipDrawOne),
                    UnoFace::DrawOne | UnoFace::WildDrawTwo
                ) | (
                    Some(UnoPendingDrawKind::FlipWildDrawTwo),
                    UnoFace::WildDrawTwo
                ) | (Some(UnoPendingDrawKind::FlipDrawFive), UnoFace::DrawFive)
                    | (
                        Some(UnoPendingDrawKind::FlipWildDrawColor),
                        UnoFace::WildDrawColor
                    )
            );
        }
        if !self.rules.action_stacking {
            return false;
        }
        match (self.pending_kind, card.face()) {
            (Some(UnoPendingDrawKind::DrawTwo), UnoFace::DrawTwo | UnoFace::ReverseDrawTwo)
            | (Some(UnoPendingDrawKind::WildDrawFour), UnoFace::WildDrawFour) => true,
            (_, UnoFace::WildNoU) => true,
            (_, UnoFace::StackOne | UnoFace::StackTwo) => card.color() == self.current_color,
            (_, UnoFace::WildStackThree | UnoFace::WildStackNumber) => true,
            (Some(UnoPendingDrawKind::DrawTwo), UnoFace::WildDrawFour) => true,
            _ => false,
        }
    }

    pub(super) fn card_allowed_in_current_state(&self, card: UnoCard) -> bool {
        if self.pending_swap.is_some() || self.skip_turns[self.current_player.0] > 0 {
            false
        } else if self.pending_skip > 0 {
            self.skip_stack_allowed(card)
        } else if self.pending_draw_active() {
            self.stack_allowed(card)
        } else {
            self.matches_top(card)
        }
    }

    pub(super) fn next_player(&self, player: UnoPlayerId) -> UnoPlayerId {
        let count = self.players.len();
        let mut next = player;
        for _ in 0..count {
            next = match self.direction {
                UnoDirection::Clockwise => UnoPlayerId((next.0 + 1) % count),
                UnoDirection::CounterClockwise => UnoPlayerId((next.0 + count - 1) % count),
            };
            if !self.players[next.0].eliminated {
                return next;
            }
        }
        player
    }

    pub(super) fn draw_cards_for(
        &mut self,
        player: UnoPlayerId,
        count: u16,
    ) -> Result<Vec<UnoCard>, GameError> {
        self.ensure_player(player)?;
        let mut cards = Vec::with_capacity(usize::from(count));
        for _ in 0..count {
            self.replenish_draw_pile();
            let card = self
                .draw_pile
                .pop_front()
                .ok_or(GameError::DrawPileExhausted)?;
            self.players[player.0].hand.push(card);
            cards.push(card);
        }
        self.players[player.0].hand.sort_by(UnoCard::display_cmp);
        Ok(cards)
    }

    pub(super) fn draw_pending_penalty(
        &mut self,
        player: UnoPlayerId,
        extra: u16,
    ) -> Result<Vec<UnoCard>, GameError> {
        if self.pending_kind == Some(UnoPendingDrawKind::FlipWildDrawColor) {
            let colors = self.pending_draw_colors.clone();
            let mut cards = Vec::new();
            for color in colors {
                cards.extend(self.draw_until_color(player, color)?);
            }
            cards.extend(self.draw_cards_for(player, extra)?);
            Ok(cards)
        } else {
            self.draw_cards_for(player, self.pending_draw.saturating_add(extra))
        }
    }

    fn replenish_draw_pile(&mut self) {
        if !self.draw_pile.is_empty() {
            return;
        }
        if self.discard_pile.len() <= 1 && self.set_aside_cards.is_empty() {
            return;
        }
        let top = self.discard_pile.pop().expect("checked above");
        let mut recycled = std::mem::take(&mut self.discard_pile);
        recycled.append(&mut self.set_aside_cards);
        recycled.reverse();
        self.draw_pile = recycled.into();
        self.discard_pile.push(top);
    }

    pub(super) fn clear_pending_draw(&mut self) {
        self.pending_draw = 0;
        self.pending_kind = None;
        self.pending_draw_source = None;
        self.pending_draw_colors.clear();
        self.challenge = None;
    }

    pub(super) const fn pending_draw_active(&self) -> bool {
        self.pending_kind.is_some()
    }

    pub(super) fn reveal_stack_number(&mut self) -> Result<(Vec<UnoCard>, u8), GameError> {
        let mut cards = Vec::new();
        let value = loop {
            self.replenish_draw_pile();
            let card = self
                .draw_pile
                .pop_front()
                .ok_or(GameError::DrawPileExhausted)?;
            cards.push(card);
            if let UnoFace::Number(value) = card.face() {
                break value;
            }
        };
        let mut discard = cards.clone();
        discard.append(&mut self.discard_pile);
        self.discard_pile = discard;
        Ok((cards, value))
    }

    pub(super) fn draw_until_color(
        &mut self,
        player: UnoPlayerId,
        color: UnoColor,
    ) -> Result<Vec<UnoCard>, GameError> {
        let mut cards = Vec::new();
        loop {
            self.replenish_draw_pile();
            let card = self
                .draw_pile
                .pop_front()
                .ok_or(GameError::DrawPileExhausted)?;
            self.players[player.0].hand.push(card);
            cards.push(card);
            if card.color() == Some(color) || self.check_mercy_elimination(player) {
                break;
            }
        }
        self.players[player.0].hand.sort_by(UnoCard::display_cmp);
        Ok(cards)
    }

    pub(super) fn check_mercy_elimination(&mut self, player: UnoPlayerId) -> bool {
        if !self.rules.is_no_mercy()
            || !self.rules.no_mercy.mercy_elimination
            || self.players[player.0].eliminated
            || self.players[player.0].hand.len() < 25
        {
            return false;
        }
        self.players[player.0].eliminated = true;
        self.elimination_order.push(player);
        self.set_aside_cards
            .append(&mut self.players[player.0].hand);
        self.uno_exposed[player.0] = false;
        self.uno_declared[player.0] = false;
        self.skip_turns[player.0] = 0;
        if self
            .players
            .iter()
            .filter(|state| !state.eliminated)
            .count()
            == 1
        {
            let winner = self
                .players
                .iter()
                .find(|state| !state.eliminated)
                .expect("one active player remains")
                .id;
            self.finish(winner);
        }
        true
    }

    pub(super) fn update_uno_after_play(&mut self, player: UnoPlayerId, declared_uno: bool) {
        if self.rules.uno_callout() && self.players[player.0].hand.len() == 1 && !declared_uno {
            self.uno_exposed[player.0] = true;
        }
    }

    pub(super) fn refresh_hand(&mut self, player: UnoPlayerId) -> Result<u16, GameError> {
        let old_hand = std::mem::take(&mut self.players[player.0].hand);
        let count = u16::try_from(old_hand.len()).unwrap_or(u16::MAX);
        let mut discard = old_hand;
        discard.append(&mut self.discard_pile);
        self.discard_pile = discard;
        self.draw_cards_for(player, count)?;
        Ok(count)
    }

    pub(super) fn pass_hands(&mut self) {
        let active = self
            .players
            .iter()
            .filter(|player| !player.eliminated)
            .map(|player| player.id)
            .collect::<Vec<_>>();
        let mut old_hands = active
            .iter()
            .map(|player| std::mem::take(&mut self.players[player.0].hand))
            .collect::<Vec<_>>();
        let count = active.len();
        for (source, hand) in old_hands.iter_mut().enumerate() {
            let target_index = match self.direction {
                UnoDirection::Clockwise => (source + 1) % count,
                UnoDirection::CounterClockwise => (source + count - 1) % count,
            };
            self.players[active[target_index].0].hand = std::mem::take(hand);
        }
    }

    pub(super) fn finish(&mut self, winner: UnoPlayerId) -> GameResult {
        self.clear_pending_draw();
        self.pending_skip = 0;
        self.pending_skip_everyone = false;
        self.pending_skip_source = None;
        self.skip_turns.fill(0);
        self.drawn_card = None;
        self.uno_exposed.fill(false);
        self.uno_declared.fill(false);
        self.jump_in_open = false;
        self.pending_swap = None;
        self.pending_finisher = None;
        let hand_scores = self
            .players
            .iter()
            .map(PlayerState::hand_score)
            .collect::<Vec<_>>();
        let placements =
            placements_with_eliminations(winner, &hand_scores, &self.elimination_order)
                .expect("the recorded elimination order contains valid players");
        let result = GameResult {
            winner,
            reference_deltas: reference_point_deltas_for_placements(&placements)
                .expect("validated placements"),
            placements,
            hand_scores,
        };
        self.phase = Phase::Finished(result.clone());
        result
    }

    pub(super) fn finish_pending_game(&mut self) -> Option<GameResult> {
        if !matches!(self.phase, Phase::Playing) {
            return None;
        }
        if self.pending_swap.is_some()
            || self.pending_draw_active()
            || self.pending_skip > 0
            || self.skip_turns.iter().any(|turns| *turns > 0)
        {
            return None;
        }
        let original = self.pending_finisher?;
        let winner = self
            .players
            .get(original.0)
            .filter(|player| !player.eliminated && player.hand.is_empty())
            .map(|player| player.id)
            .or_else(|| {
                self.players
                    .iter()
                    .find(|player| !player.eliminated && player.hand.is_empty())
                    .map(|player| player.id)
            })?;
        Some(self.finish(winner))
    }
}
