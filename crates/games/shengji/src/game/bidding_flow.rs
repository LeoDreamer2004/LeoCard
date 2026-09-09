use super::{
    ActionOutcome, BottomFlipMatch, BottomFlipReveal, GameError, GameState, Phase, next_player,
};
use crate::{BidError, BidState, ShengjiPlayerId, ShengjiRuleSet, ShengjiTrump};

impl GameState {
    pub fn close_bidding_and_take_kitty(&mut self) -> Result<ActionOutcome, GameError> {
        if self.phase != Phase::BiddingGrace {
            return Err(GameError::WrongPhase);
        }
        let trump = match self.bidding.close() {
            Ok(trump) => trump.with_constant_trump(self.rules.constant_trump),
            Err(BidError::NoDeclaration) => {
                if self.rules.power_outage_dealer && !self.power_outage_used {
                    let dealer = next_player(self.bidding_dealer);
                    self.power_outage_used = true;
                    self.fixed_dealer = Some(dealer);
                    self.bidding_dealer = dealer;
                    self.dealer = Some(dealer);
                    self.bidding = BidState::new_with_rules(
                        self.teams.level(dealer.team()),
                        self.rules.deck_count,
                        self.rules.bid_with_joker,
                    );
                    return Ok(ActionOutcome::PowerOutageDealerChanged { dealer });
                }
                if self.rules.bottom_flip {
                    self.bottom_flip_index = 0;
                    self.bottom_flip_winner = None;
                    self.phase = Phase::BottomFlipping;
                    return Ok(ActionOutcome::BottomFlipStarted);
                }
                self.phase = Phase::RedealRequired;
                return Err(GameError::RedealRequired);
            }
            Err(error) => return Err(error.into()),
        };
        let dealer = self
            .fixed_dealer
            .unwrap_or_else(|| self.bidding.current().unwrap().player);
        self.bidding_dealer = dealer;
        self.take_kitty_for_dealer(dealer, trump);
        Ok(ActionOutcome::DealerTookKitty { dealer })
    }

    pub fn flip_next_bottom_card(&mut self) -> Result<ActionOutcome, GameError> {
        if self.phase != Phase::BottomFlipping {
            return Err(GameError::WrongPhase);
        }
        let Some(card) = self.deck.get(self.bottom_flip_index).copied() else {
            self.phase = Phase::RedealRequired;
            return Err(GameError::RedealRequired);
        };
        self.bottom_flip_index += 1;
        let matches = self
            .players
            .iter()
            .filter_map(|player| {
                let cards = player
                    .hand
                    .iter()
                    .copied()
                    .filter(|candidate| candidate.same_face(card))
                    .collect::<Vec<_>>();
                (!cards.is_empty()).then_some(BottomFlipMatch {
                    player: player.id,
                    cards,
                })
            })
            .collect::<Vec<_>>();
        let dealer = select_bottom_flip_dealer(&matches, self.bidding_dealer);
        if let Some(dealer) = dealer {
            let trump = ShengjiTrump::new(self.teams.level(dealer.team()), card.suit())
                .expect("扳底只会从完整牌堆中翻出合法牌")
                .with_constant_trump(self.rules.constant_trump);
            self.fixed_dealer = Some(dealer);
            self.bidding_dealer = dealer;
            self.dealer = Some(dealer);
            self.trump = Some(trump);
            self.dealer_from_bottom_flip = true;
            self.bottom_flip_winner = Some((dealer, trump));
        } else if self.bottom_flip_index == self.deck.len() {
            self.phase = Phase::RedealRequired;
        }
        Ok(ActionOutcome::BottomCardRevealed(BottomFlipReveal {
            card,
            matches,
            dealer,
        }))
    }

    pub fn complete_bottom_flip(&mut self) -> Result<ActionOutcome, GameError> {
        if self.phase != Phase::BottomFlipping {
            return Err(GameError::WrongPhase);
        }
        let Some((dealer, trump)) = self.bottom_flip_winner.take() else {
            return Err(GameError::WrongPhase);
        };
        self.take_kitty_for_dealer(dealer, trump);
        Ok(ActionOutcome::DealerTookKitty { dealer })
    }

    fn take_kitty_for_dealer(&mut self, dealer: ShengjiPlayerId, trump: ShengjiTrump) {
        self.kitty = self.deck.drain(..).collect();
        debug_assert_eq!(self.kitty.len(), self.rules.kitty_size());
        self.players[usize::from(dealer.0)]
            .hand
            .extend(self.kitty.iter().copied());
        self.trump = Some(trump);
        self.dealer = Some(dealer);
        self.phase = Phase::Burying;
    }
}

pub(super) fn select_bottom_flip_dealer(
    matches: &[BottomFlipMatch],
    current_dealer: ShengjiPlayerId,
) -> Option<ShengjiPlayerId> {
    if matches.len() == 1 {
        return Some(matches[0].player);
    }
    if matches.is_empty() {
        return None;
    }
    let mut team_totals = [0_usize; 2];
    for matched in matches {
        team_totals[usize::from(matched.player.team().0)] += matched.cards.len();
    }
    if team_totals[0] == team_totals[1] {
        return None;
    }
    let winning_team = usize::from(team_totals[1] > team_totals[0]);
    let most_cards = matches
        .iter()
        .filter(|matched| usize::from(matched.player.team().0) == winning_team)
        .map(|matched| matched.cards.len())
        .max()?;
    matches
        .iter()
        .filter(|matched| {
            usize::from(matched.player.team().0) == winning_team
                && matched.cards.len() == most_cards
        })
        .min_by_key(|matched| {
            (matched.player.0 + ShengjiRuleSet::PLAYER_COUNT as u8 - current_dealer.0)
                % ShengjiRuleSet::PLAYER_COUNT as u8
        })
        .map(|matched| matched.player)
}
