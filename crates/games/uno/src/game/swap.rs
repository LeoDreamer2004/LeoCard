use crate::{UnoCard, UnoColor, UnoFace};
use leocard_game_common::PlayerId;

use super::{
    PendingSwapState, UnoActionOutcome, UnoGameError, UnoGameState, remove_card, swap_player_hands,
};

impl UnoGameState {
    pub fn choose_swap_one_target(
        &mut self,
        player: PlayerId,
        target: PlayerId,
        taken_index: usize,
    ) -> Result<UnoActionOutcome, UnoGameError> {
        self.ensure_turn(player)?;
        self.ensure_player(target)?;
        let Some(PendingSwapState::SwapOneTarget {
            player: expected,
            declared_uno,
        }) = self.pending_swap
        else {
            return Err(UnoGameError::NoSwapEffect);
        };
        if player != expected {
            return Err(UnoGameError::NotPlayersTurn {
                expected,
                actual: player,
            });
        }
        if target == player || self.players[target.index()].eliminated {
            return Err(UnoGameError::InvalidSwapTargets);
        }
        let Some(card) = self.players[target.index()].hand.get(taken_index).copied() else {
            return Err(UnoGameError::InvalidSwapTargets);
        };
        self.players[target.index()].hand.remove(taken_index);
        self.players[player.index()].hand.push(card);
        self.players[player.index()]
            .hand
            .sort_by(UnoCard::display_cmp);
        self.pending_swap = Some(PendingSwapState::SwapOneGive {
            player,
            target,
            declared_uno,
        });
        Ok(UnoActionOutcome::SwapOneCardTaken { player, target })
    }

    pub fn choose_seven_swap_target(
        &mut self,
        player: PlayerId,
        target: PlayerId,
    ) -> Result<UnoActionOutcome, UnoGameError> {
        self.ensure_turn(player)?;
        self.ensure_player(target)?;
        let Some(PendingSwapState::SevenSwap { player: expected }) = self.pending_swap else {
            return Err(UnoGameError::NoSwapEffect);
        };
        if player != expected {
            return Err(UnoGameError::NotPlayersTurn {
                expected,
                actual: player,
            });
        }
        if target == player || self.players[target.index()].eliminated {
            return Err(UnoGameError::InvalidSwapTargets);
        }
        swap_player_hands(&mut self.players, player, target);
        for selected in [player, target] {
            self.uno_exposed[selected.index()] = false;
            self.uno_declared[selected.index()] = false;
        }
        self.pending_swap = None;
        let next_player = self.next_player(player);
        self.current_player = next_player;
        self.finish_pending_game();
        Ok(UnoActionOutcome::HandsTraded {
            player,
            first: player,
            second: target,
        })
    }

    pub fn give_swap_one_card(
        &mut self,
        player: PlayerId,
        card: UnoCard,
    ) -> Result<UnoActionOutcome, UnoGameError> {
        self.ensure_turn(player)?;
        let Some(PendingSwapState::SwapOneGive {
            player: expected,
            target,
            declared_uno,
        }) = self.pending_swap
        else {
            return Err(UnoGameError::NoSwapEffect);
        };
        if player != expected {
            return Err(UnoGameError::NotPlayersTurn {
                expected,
                actual: player,
            });
        }
        if !self.players[player.index()].hand.contains(&card) {
            return Err(UnoGameError::CardNotInHand(card));
        }
        remove_card(&mut self.players[player.index()].hand, card);
        self.players[target.index()].hand.push(card);
        self.players[target.index()]
            .hand
            .sort_by(UnoCard::display_cmp);
        self.pending_swap = None;
        self.update_uno_after_play(player, declared_uno);
        let next_player = self.next_player(player);
        self.current_player = next_player;
        self.jump_in_open = false;
        self.finish_pending_game();
        Ok(UnoActionOutcome::SwapOneCompleted {
            player,
            target,
            next_player,
        })
    }

    pub fn force_trade_hands(
        &mut self,
        player: PlayerId,
        first: PlayerId,
        second: PlayerId,
    ) -> Result<UnoActionOutcome, UnoGameError> {
        self.ensure_turn(player)?;
        self.ensure_player(first)?;
        self.ensure_player(second)?;
        let Some(PendingSwapState::ForceTrade { player: expected }) = self.pending_swap else {
            return Err(UnoGameError::NoSwapEffect);
        };
        if player != expected {
            return Err(UnoGameError::NotPlayersTurn {
                expected,
                actual: player,
            });
        }
        if first == second
            || self.players[first.index()].eliminated
            || self.players[second.index()].eliminated
        {
            return Err(UnoGameError::InvalidSwapTargets);
        }
        swap_player_hands(&mut self.players, first, second);
        for target in [first, second] {
            self.uno_exposed[target.index()] = false;
            self.uno_declared[target.index()] = false;
        }
        self.pending_swap = Some(PendingSwapState::ChooseColor { player });
        Ok(UnoActionOutcome::HandsTraded {
            player,
            first,
            second,
        })
    }

    /// 一次提交一张普通牌，或在抢出规则下提交两张完全相同的彩色牌。
    /// 双牌通过克隆状态原子结算，第二张失败时不会留下只打出第一张的半成品。
    pub fn play_cards(
        &mut self,
        player: PlayerId,
        cards: &[UnoCard],
        chosen_color: Option<UnoColor>,
    ) -> Result<UnoActionOutcome, UnoGameError> {
        let [first, second] = cards else {
            return match cards {
                [card] => self.play_card(player, *card, chosen_color),
                _ => Err(UnoGameError::CannotPlayTogether),
            };
        };
        if !self.jump_in_enabled()
            || (!self.action_stacking_enabled() && !matches!(first.face(), UnoFace::Number(_)))
            || chosen_color.is_some()
            || first == second
            || first.color().is_none()
            || first.face().is_extension()
            || first.color() != second.color()
            || first.face() != second.face()
            || self
                .uno_declared
                .get(player.index())
                .copied()
                .unwrap_or(false)
            || !self
                .players
                .get(player.index())
                .is_some_and(|state| state.hand.contains(first) && state.hand.contains(second))
        {
            return Err(UnoGameError::CannotPlayTogether);
        }

        if first.face() == UnoFace::Flip {
            return self.play_flip_pair(player, *first, *second);
        }

        let mut staged = self.clone();
        staged.play_card(player, *first, None)?;
        let outcome = if staged.current_player == player {
            staged.play_card(player, *second, None)?
        } else {
            staged.jump_in(player, *second)?
        };
        *self = staged;
        Ok(outcome)
    }

    fn play_flip_pair(
        &mut self,
        player: PlayerId,
        first: UnoCard,
        second: UnoCard,
    ) -> Result<UnoActionOutcome, UnoGameError> {
        if !self.can_play(player, first)
            || self.drawn_card.is_some()
            || self.pending_draw_active()
            || self.pending_skip > 0
            || self.skip_turns[player.index()] > 0
        {
            return Err(UnoGameError::CannotPlayTogether);
        }
        remove_card(&mut self.players[player.index()].hand, first);
        remove_card(&mut self.players[player.index()].hand, second);
        self.discard_pile.extend([first, second]);
        self.current_color = second.color();
        self.uno_exposed[player.index()] = false;
        self.uno_declared[player.index()] = false;
        if self.players[player.index()].hand.is_empty() {
            self.pending_finisher.get_or_insert(player);
        } else {
            self.update_uno_after_play(player, false);
        }
        self.jump_in_open = false;
        let next_player = self.next_player(player);
        self.current_player = next_player;
        self.finish_pending_game();
        Ok(UnoActionOutcome::Played {
            player,
            card: second,
            next_player,
            effect: None,
        })
    }
}
