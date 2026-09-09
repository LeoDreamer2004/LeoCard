use super::{
    ActionOutcome, BottomCopyState, FiveTrumpCrossingStage, FiveTrumpCrossingState, GameError,
    GameState, Phase, next_player, remove_cards, validate_cards_owned,
};
use crate::{ShengjiCard, ShengjiPlayerId, ShengjiRuleSet, ShengjiTrump};
use std::array;
use std::mem;

impl GameState {
    pub fn bury(
        &mut self,
        player: ShengjiPlayerId,
        cards: &[ShengjiCard],
    ) -> Result<ActionOutcome, GameError> {
        let dealer = self.dealer.unwrap();
        let bottom_copy_bury = match self.phase {
            Phase::Burying => {
                if player != dealer {
                    return Err(GameError::NotDealer);
                }
                false
            }
            Phase::BottomCopyBurying => {
                if self
                    .bottom_copy
                    .as_ref()
                    .and_then(BottomCopyState::bottom_holder)
                    != Some(player)
                {
                    return Err(GameError::NotBottomCopyPlayer);
                }
                true
            }
            _ => return Err(GameError::WrongPhase),
        };
        if cards.len() != self.rules.kitty_size() {
            return Err(GameError::WrongBuryCount {
                expected: self.rules.kitty_size(),
                actual: cards.len(),
            });
        }
        validate_cards_owned(cards, &self.players[usize::from(player.0)].hand)?;
        remove_cards(&mut self.players[usize::from(player.0)].hand, cards);
        self.buried = cards.to_vec();
        self.bottom_burier = Some(player);
        if bottom_copy_bury {
            let state = self.bottom_copy.as_mut().unwrap();
            state.bottom_holder = None;
            state.last_burier = player;
            let inquiry_complete = !self.advance_bottom_copy_inquiry();
            return Ok(ActionOutcome::BottomCopyBuryComplete {
                player,
                inquiry_complete,
            });
        }
        if self.rules.bottom_copy
            && !self.dealer_from_bottom_flip
            && self.start_bottom_copy_inquiry()
        {
            return Ok(ActionOutcome::BuryComplete { leader: dealer });
        }
        self.finish_after_burying();
        Ok(ActionOutcome::BuryComplete { leader: dealer })
    }

    pub fn choose_bottom_copy(
        &mut self,
        player: ShengjiPlayerId,
        cards: Option<&[ShengjiCard]>,
    ) -> Result<ActionOutcome, GameError> {
        if self.phase != Phase::BottomCopying {
            return Err(GameError::WrongPhase);
        }
        if self
            .bottom_copy
            .as_ref()
            .is_none_or(|state| state.current != Some(player) || state.last_burier == player)
        {
            return Err(GameError::NotBottomCopyPlayer);
        }
        if let Some(cards) = cards {
            let hand = self.players[usize::from(player.0)].hand.clone();
            let declaration = self
                .bidding
                .counter_after_close(player, cards, &hand)?
                .clone();
            self.trump = Some(
                ShengjiTrump::new(self.bidding.level(), declaration.trump.trump_suit())
                    .expect("抄底反主保持普通级牌")
                    .with_constant_trump(self.rules.constant_trump),
            );
            let old_bottom = mem::take(&mut self.buried);
            self.bottom_burier = None;
            self.players[usize::from(player.0)].hand.extend(old_bottom);
            let state = self.bottom_copy.as_mut().unwrap();
            state.current = None;
            state.bottom_holder = Some(player);
            state.remaining_without_copy = ShengjiRuleSet::PLAYER_COUNT as u8;
            self.phase = Phase::BottomCopyBurying;
            return Ok(ActionOutcome::BottomCopyDecision {
                player,
                copied: true,
                inquiry_complete: false,
            });
        }
        self.bottom_copy.as_mut().unwrap().current = None;
        let inquiry_complete = !self.advance_bottom_copy_inquiry();
        Ok(ActionOutcome::BottomCopyDecision {
            player,
            copied: false,
            inquiry_complete,
        })
    }

    fn start_bottom_copy_inquiry(&mut self) -> bool {
        let dealer = self.dealer.unwrap();
        self.bottom_copy = Some(BottomCopyState {
            next_player: next_player(dealer),
            remaining_without_copy: ShengjiRuleSet::PLAYER_COUNT as u8,
            current: None,
            bottom_holder: None,
            last_burier: dealer,
        });
        self.advance_bottom_copy_inquiry()
    }

    fn advance_bottom_copy_inquiry(&mut self) -> bool {
        loop {
            if self
                .bottom_copy
                .as_ref()
                .is_some_and(|state| state.remaining_without_copy == 0)
            {
                self.bottom_copy = None;
                self.finish_after_burying();
                return false;
            }
            let player = {
                let state = self.bottom_copy.as_mut().unwrap();
                let player = state.next_player;
                state.next_player = next_player(player);
                state.remaining_without_copy -= 1;
                player
            };
            if self
                .bottom_copy
                .as_ref()
                .is_some_and(|state| state.last_burier == player)
                || self
                    .bidding
                    .current()
                    .is_some_and(|bid| bid.player == player)
            {
                continue;
            }
            let hand = &self.players[usize::from(player.0)].hand;
            if !self.bidding.counter_options(player, hand).is_empty() {
                self.bottom_copy.as_mut().unwrap().current = Some(player);
                self.phase = Phase::BottomCopying;
                self.current_player = None;
                return true;
            }
        }
    }

    fn finish_after_burying(&mut self) {
        let trump = self.trump.expect("庄家埋底前已经确定主牌");
        if self.rules.five_trump_crossing && trump.suit.is_some() {
            let eligible = array::from_fn(|index| {
                self.players[index]
                    .hand
                    .iter()
                    .filter(|card| trump.is_trump(**card))
                    .count()
                    <= 5
            });
            if eligible.iter().any(|eligible| *eligible) {
                self.five_trump_crossing = Some(FiveTrumpCrossingState {
                    stage: FiveTrumpCrossingStage::Deciding,
                    eligible,
                    declined: [false; ShengjiRuleSet::PLAYER_COUNT],
                    outgoing: array::from_fn(|_| None),
                    returned: array::from_fn(|_| None),
                });
                self.phase = Phase::FiveTrumpCrossing;
                self.current_player = None;
            } else {
                self.start_playing();
            }
        } else {
            self.start_playing();
        }
    }
}
