use super::{ActionOutcome, GameError, GameState, Phase};
use crate::{ShengjiCard, ShengjiPlayerId, ShengjiRuleSet};

impl GameState {
    pub fn deal_next(&mut self) -> Result<ActionOutcome, GameError> {
        if self.phase != Phase::Dealing {
            return Err(GameError::WrongPhase);
        }
        let dealt = self
            .players
            .iter()
            .map(|player| player.hand.len())
            .sum::<usize>();
        if dealt == ShengjiRuleSet::PLAYER_COUNT * self.rules.hand_size() {
            self.phase = Phase::BiddingGrace;
            return Ok(ActionOutcome::DealComplete);
        }
        let player = ShengjiPlayerId(
            (usize::from(self.first_recipient.0) + dealt) as u8
                % ShengjiRuleSet::PLAYER_COUNT as u8,
        );
        let card = self
            .deck
            .pop_front()
            .expect("validated deck has enough cards");
        self.players[usize::from(player.0)].hand.push(card);
        Ok(ActionOutcome::CardDealt { player, card })
    }

    pub fn deal_all(&mut self) -> Result<(), GameError> {
        while self.phase == Phase::Dealing {
            self.deal_next()?;
        }
        Ok(())
    }

    pub fn declare(
        &mut self,
        player: ShengjiPlayerId,
        cards: &[ShengjiCard],
    ) -> Result<ActionOutcome, GameError> {
        if !matches!(self.phase, Phase::Dealing | Phase::BiddingGrace) {
            return Err(GameError::WrongPhase);
        }
        let hand = self.player(player)?.hand.clone();
        self.bidding.declare(player, cards, &hand)?;
        if self.fixed_dealer.is_none() {
            self.dealer = self.bidding.current().map(|declaration| declaration.player);
        }
        Ok(ActionOutcome::DeclarationChanged)
    }
}
