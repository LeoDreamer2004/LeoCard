use super::{ShengjiActionOutcome, ShengjiGameError, ShengjiGameState, ShengjiPhase};
use crate::ShengjiCard;
use leocard_game_common::PlayerId;

impl ShengjiGameState {
    pub fn deal_next(&mut self) -> Result<ShengjiActionOutcome, ShengjiGameError> {
        if self.phase != ShengjiPhase::Dealing {
            return Err(ShengjiGameError::WrongPhase);
        }
        let dealt = self
            .players
            .iter()
            .map(|player| player.hand.len())
            .sum::<usize>();
        if dealt == usize::from(self.rules.player_count()) * self.rules.hand_size() {
            self.phase = ShengjiPhase::BiddingGrace;
            return Ok(ShengjiActionOutcome::DealComplete);
        }
        let player = PlayerId(
            (usize::from(self.first_recipient.0) + dealt) as u8 % self.rules.player_count(),
        );
        let card = self
            .deck
            .pop_front()
            .expect("validated deck has enough cards");
        self.players[usize::from(player.0)].hand.push(card);
        Ok(ShengjiActionOutcome::CardDealt { player, card })
    }

    pub fn deal_all(&mut self) -> Result<(), ShengjiGameError> {
        while self.phase == ShengjiPhase::Dealing {
            self.deal_next()?;
        }
        Ok(())
    }

    pub fn declare(
        &mut self,
        player: PlayerId,
        cards: &[ShengjiCard],
    ) -> Result<ShengjiActionOutcome, ShengjiGameError> {
        if !matches!(
            self.phase,
            ShengjiPhase::Dealing | ShengjiPhase::BiddingGrace
        ) {
            return Err(ShengjiGameError::WrongPhase);
        }
        let hand = self.player(player)?.hand.clone();
        self.bidding.declare(player, cards, &hand)?;
        if self.fixed_dealer.is_none() {
            self.dealer = self.bidding.current().map(|declaration| declaration.player);
        }
        Ok(ShengjiActionOutcome::DeclarationChanged)
    }
}
