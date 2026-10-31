use super::{
    DoudizhuAction, DoudizhuGameError, DoudizhuGameState, DoudizhuPhase, DoudizhuResultView,
};
use leocard_game_common::PlayerId;

use crate::classify;

impl DoudizhuGameState {
    pub fn act(
        &mut self,
        player: PlayerId,
        action: DoudizhuAction,
    ) -> Result<(), DoudizhuGameError> {
        if usize::from(player) >= self.hands.len() {
            return Err(DoudizhuGameError::InvalidPlayer);
        }
        if matches!(self.phase, DoudizhuPhase::Finished | DoudizhuPhase::Redeal) {
            return Err(DoudizhuGameError::WrongPhase);
        }
        if player != self.current {
            return Err(DoudizhuGameError::NotYourTurn);
        }
        match &action {
            DoudizhuAction::Bid(score) => self.bid(player, *score)?,
            DoudizhuAction::Play(cards) => {
                if self.phase != DoudizhuPhase::Playing {
                    return Err(DoudizhuGameError::WrongPhase);
                }
                let play = classify(cards, self.rules).ok_or(DoudizhuGameError::InvalidPattern)?;
                let hand = &mut self.hands[usize::from(player)];
                if !cards.iter().all(|card| hand.contains(card)) {
                    return Err(DoudizhuGameError::CardNotInHand);
                }
                if self
                    .table
                    .as_ref()
                    .is_some_and(|(_, previous)| !play.beats(previous))
                {
                    return Err(DoudizhuGameError::CannotBeat);
                }
                hand.retain(|card| !cards.contains(card));
                self.played.extend_from_slice(cards);
                self.play_counts[usize::from(player)] += 1;
                if play.doubles(self.rules.mode) {
                    self.multiplier = (self.multiplier * 2).min(1 << 20);
                }
                self.table = Some((player, play));
                self.passes = 0;
                if hand.is_empty() {
                    self.finish(player);
                } else {
                    self.current = PlayerId((player.0 + 1) % self.rules.mode.players());
                }
            }
            DoudizhuAction::Pass => {
                if self.phase != DoudizhuPhase::Playing {
                    return Err(DoudizhuGameError::WrongPhase);
                }
                let Some((winner, _)) = &self.table else {
                    return Err(DoudizhuGameError::MustPlay);
                };
                self.passes += 1;
                if self.passes == self.rules.mode.players() - 1 {
                    self.current = *winner;
                    self.table = None;
                    self.passes = 0;
                    self.last_actions.fill(None);
                } else {
                    self.current = PlayerId((player.0 + 1) % self.rules.mode.players());
                }
            }
        }
        if self.phase == DoudizhuPhase::Bidding || self.table.is_some() {
            self.last_actions[usize::from(player)] = Some(action);
        }
        Ok(())
    }

    fn bid(&mut self, player: PlayerId, score: u8) -> Result<(), DoudizhuGameError> {
        if self.phase != DoudizhuPhase::Bidding {
            return Err(DoudizhuGameError::WrongPhase);
        }
        if score > 3 || (score != 0 && score <= self.highest_bid) {
            return Err(DoudizhuGameError::InvalidBid);
        }
        self.bids[usize::from(player)] = Some(score);
        if score != 0 {
            self.highest_bid = score;
            self.landlord = Some(player);
        }
        if score == 3 || self.bids.iter().all(Option::is_some) {
            if let Some(landlord) = self.landlord {
                self.hands[usize::from(landlord)].extend_from_slice(&self.kitty);
                self.hands[usize::from(landlord)].sort_by_key(|c| (c.rank(), c.0));
                self.phase = DoudizhuPhase::Playing;
                self.current = landlord;
                self.last_actions.fill(None);
            } else {
                self.phase = DoudizhuPhase::Redeal;
            }
        } else {
            self.current = PlayerId((player.0 + 1) % self.rules.mode.players());
        }
        Ok(())
    }

    fn finish(&mut self, winner: PlayerId) {
        let landlord = self.landlord.expect("playing requires a landlord");
        let landlord_won = winner == landlord;
        let spring = if landlord_won {
            self.play_counts
                .iter()
                .enumerate()
                .all(|(i, &n)| i == usize::from(landlord) || n == 0)
        } else {
            self.play_counts[usize::from(landlord)] <= 1
        };
        if spring {
            self.multiplier = (self.multiplier * 2).min(1 << 20);
        }
        let stake = i32::from(self.highest_bid) * self.multiplier as i32;
        let sign = if landlord_won { 1 } else { -1 };
        let scores = (0..self.rules.mode.players())
            .map(|p| {
                if PlayerId(p) == landlord {
                    sign * stake * i32::from(self.rules.mode.players() - 1)
                } else {
                    -sign * stake
                }
            })
            .collect();
        self.result = Some(DoudizhuResultView {
            winner,
            landlord_won,
            spring,
            scores,
        });
        self.phase = DoudizhuPhase::Finished;
    }
}
