use super::{Action, GameError, GameState, Phase, ResultView};
use crate::classify;

impl GameState {
    pub fn act(&mut self, player: u8, action: Action) -> Result<(), GameError> {
        if usize::from(player) >= self.hands.len() {
            return Err(GameError::InvalidPlayer);
        }
        if matches!(self.phase, Phase::Finished | Phase::Redeal) {
            return Err(GameError::WrongPhase);
        }
        if player != self.current {
            return Err(GameError::NotYourTurn);
        }
        match &action {
            Action::Bid(score) => self.bid(player, *score)?,
            Action::Play(cards) => {
                if self.phase != Phase::Playing {
                    return Err(GameError::WrongPhase);
                }
                let play = classify(cards, self.rules.mode).ok_or(GameError::InvalidPattern)?;
                let hand = &mut self.hands[usize::from(player)];
                if !cards.iter().all(|card| hand.contains(card)) {
                    return Err(GameError::CardNotInHand);
                }
                if self
                    .table
                    .as_ref()
                    .is_some_and(|(_, previous)| !play.beats(previous))
                {
                    return Err(GameError::CannotBeat);
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
                    self.current = (player + 1) % self.rules.mode.players();
                }
            }
            Action::Pass => {
                if self.phase != Phase::Playing {
                    return Err(GameError::WrongPhase);
                }
                let Some((winner, _)) = &self.table else {
                    return Err(GameError::MustPlay);
                };
                self.passes += 1;
                if self.passes == self.rules.mode.players() - 1 {
                    self.current = *winner;
                    self.table = None;
                    self.passes = 0;
                    self.last_actions.fill(None);
                } else {
                    self.current = (player + 1) % self.rules.mode.players();
                }
            }
        }
        if self.phase == Phase::Bidding || self.table.is_some() {
            self.last_actions[usize::from(player)] = Some(action);
        }
        Ok(())
    }

    fn bid(&mut self, player: u8, score: u8) -> Result<(), GameError> {
        if self.phase != Phase::Bidding {
            return Err(GameError::WrongPhase);
        }
        if score > 3 || (score != 0 && score <= self.highest_bid) {
            return Err(GameError::InvalidBid);
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
                self.phase = Phase::Playing;
                self.current = landlord;
                self.last_actions.fill(None);
            } else {
                self.phase = Phase::Redeal;
            }
        } else {
            self.current = (player + 1) % self.rules.mode.players();
        }
        Ok(())
    }

    fn finish(&mut self, winner: u8) {
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
                if p == landlord {
                    sign * stake * i32::from(self.rules.mode.players() - 1)
                } else {
                    -sign * stake
                }
            })
            .collect();
        self.result = Some(ResultView {
            winner,
            landlord_won,
            spring,
            scores,
        });
        self.phase = Phase::Finished;
    }
}
