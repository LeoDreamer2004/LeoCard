use crate::{UnoCard, UnoColor, UnoFace, UnoFlipSide};
use leocard_game_common::PlayerId;

use std::mem;

use super::{UnoDirection, UnoGameError, UnoGameState, UnoPendingDrawKind, UnoPhase, faces_match};

impl UnoGameState {
    pub(super) fn apply_starting_card(&mut self) -> Result<(), UnoGameError> {
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
                    self.skip_turns[self.current_player.index()] = 1;
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
                    self.skip_turns[self.current_player.index()] = 1;
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

    pub(super) fn ensure_playing(&self) -> Result<(), UnoGameError> {
        if matches!(self.phase, UnoPhase::Finished(_)) {
            Err(UnoGameError::GameAlreadyFinished)
        } else {
            Ok(())
        }
    }

    pub(super) fn ensure_player(&self, player: PlayerId) -> Result<(), UnoGameError> {
        if player.index() < self.players.len() {
            Ok(())
        } else {
            Err(UnoGameError::InvalidPlayer(player))
        }
    }

    pub(super) fn ensure_turn(&self, player: PlayerId) -> Result<(), UnoGameError> {
        self.ensure_playing()?;
        self.ensure_player(player)?;
        if player == self.current_player && !self.players[player.index()].eliminated {
            Ok(())
        } else {
            Err(UnoGameError::NotPlayersTurn {
                expected: self.current_player,
                actual: player,
            })
        }
    }

    pub(super) fn ensure_skip_resolved(&self, player: PlayerId) -> Result<(), UnoGameError> {
        if self.pending_skip > 0 || self.skip_turns[player.index()] > 0 {
            Err(UnoGameError::MustResolveSkip)
        } else {
            Ok(())
        }
    }

    pub(super) fn ensure_swap_resolved(&self) -> Result<(), UnoGameError> {
        if self.pending_swap.is_some() {
            Err(UnoGameError::MustResolveSwapEffect)
        } else {
            Ok(())
        }
    }

    pub(super) fn ensure_uno_followup_resolved(
        &self,
        player: PlayerId,
    ) -> Result<(), UnoGameError> {
        if self.uno_declared[player.index()] {
            Err(UnoGameError::MustPlayAfterUno)
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
        self.rules.action_stacking_enabled()
    }

    pub(super) const fn skip_draw_penalty_enabled(&self) -> bool {
        if self.rules.is_flip() {
            self.rules.flip.skip_draw_penalty
        } else {
            self.rules.is_classic() && self.rules.skip_draw_penalty
        }
    }

    pub(super) const fn jump_in_enabled(&self) -> bool {
        self.rules.jump_in_enabled()
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
        if self.pending_swap.is_some() || self.skip_turns[self.current_player.index()] > 0 {
            false
        } else if self.pending_skip > 0 {
            self.skip_stack_allowed(card)
        } else if self.pending_draw_active() {
            self.stack_allowed(card)
        } else {
            self.sudden_death_free_play() || self.matches_top(card)
        }
    }

    pub(super) fn sudden_death_active(&self) -> bool {
        self.rules.sudden_death_enabled()
            && self
                .players
                .iter()
                .filter(|player| !player.eliminated)
                .count()
                == 2
    }

    pub(super) fn sudden_death_free_play(&self) -> bool {
        self.sudden_death_active() && self.sudden_death_misses >= 2
    }

    pub(super) fn sudden_death_missed(&mut self, player: PlayerId) -> PlayerId {
        self.sudden_death_misses = self.sudden_death_misses.saturating_add(1).min(2);
        if self.sudden_death_free_play() {
            player
        } else {
            self.next_player(player)
        }
    }

    pub(super) fn next_player(&self, player: PlayerId) -> PlayerId {
        let count = self.players.len();
        let mut next = player;
        for _ in 0..count {
            next = match self.direction {
                UnoDirection::Clockwise => PlayerId::from_index((next.index() + 1) % count),
                UnoDirection::CounterClockwise => {
                    PlayerId::from_index((next.index() + count - 1) % count)
                }
            };
            if !self.players[next.index()].eliminated {
                return next;
            }
        }
        player
    }

    pub(super) fn draw_cards_for(
        &mut self,
        player: PlayerId,
        count: u16,
    ) -> Result<Vec<UnoCard>, UnoGameError> {
        self.ensure_player(player)?;
        let mut cards = Vec::with_capacity(usize::from(count));
        for _ in 0..count {
            self.replenish_draw_pile();
            let Some(card) = self.draw_pile.pop_front() else {
                if self.sudden_death_active() {
                    break;
                }
                return Err(UnoGameError::DrawPileExhausted);
            };
            self.players[player.index()].hand.push(card);
            cards.push(card);
        }
        self.players[player.index()]
            .hand
            .sort_by(UnoCard::display_cmp);
        Ok(cards)
    }

    pub(super) fn draw_pending_penalty(
        &mut self,
        player: PlayerId,
        extra: u16,
    ) -> Result<Vec<UnoCard>, UnoGameError> {
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
        if self.sudden_death_active() {
            return;
        }
        if self.discard_pile.len() <= 1 && self.set_aside_cards.is_empty() {
            return;
        }
        let top = self.discard_pile.pop().expect("checked above");
        let mut recycled = mem::take(&mut self.discard_pile);
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

    pub(super) fn reveal_stack_number(&mut self) -> Result<(Vec<UnoCard>, u8), UnoGameError> {
        let mut cards = Vec::new();
        let value = loop {
            self.replenish_draw_pile();
            let card = self
                .draw_pile
                .pop_front()
                .ok_or(UnoGameError::DrawPileExhausted)?;
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
        player: PlayerId,
        color: UnoColor,
    ) -> Result<Vec<UnoCard>, UnoGameError> {
        let mut cards = Vec::new();
        loop {
            self.replenish_draw_pile();
            let Some(card) = self.draw_pile.pop_front() else {
                if self.sudden_death_active() {
                    break;
                }
                return Err(UnoGameError::DrawPileExhausted);
            };
            self.players[player.index()].hand.push(card);
            cards.push(card);
            if card.color() == Some(color) || self.check_mercy_elimination(player) {
                break;
            }
        }
        self.players[player.index()]
            .hand
            .sort_by(UnoCard::display_cmp);
        Ok(cards)
    }

    pub(super) fn check_mercy_elimination(&mut self, player: PlayerId) -> bool {
        if !self.rules.is_no_mercy()
            || !self.rules.no_mercy.mercy_elimination
            || self.players[player.index()].eliminated
            || self.players[player.index()].hand.len() < 25
        {
            return false;
        }
        self.players[player.index()].eliminated = true;
        self.elimination_order.push(player);
        self.set_aside_cards
            .append(&mut self.players[player.index()].hand);
        self.uno_exposed[player.index()] = false;
        self.uno_declared[player.index()] = false;
        self.skip_turns[player.index()] = 0;
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

    pub(super) fn update_uno_after_play(&mut self, player: PlayerId, declared_uno: bool) {
        if self.rules.uno_callout() && self.players[player.index()].hand.len() == 1 && !declared_uno
        {
            self.uno_exposed[player.index()] = true;
        }
    }

    pub(super) fn refresh_hand(&mut self, player: PlayerId) -> Result<u16, UnoGameError> {
        let old_hand = mem::take(&mut self.players[player.index()].hand);
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
            .map(|player| mem::take(&mut self.players[player.index()].hand))
            .collect::<Vec<_>>();
        let count = active.len();
        for (source, hand) in old_hands.iter_mut().enumerate() {
            let target_index = match self.direction {
                UnoDirection::Clockwise => (source + 1) % count,
                UnoDirection::CounterClockwise => (source + count - 1) % count,
            };
            self.players[active[target_index].index()].hand = mem::take(hand);
        }
    }
}
