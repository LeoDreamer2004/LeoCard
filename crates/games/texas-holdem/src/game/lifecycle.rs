use super::{
    TexasBlindKind, TexasGameError, TexasGameState, TexasPhase, TexasStreet, validate_deck,
};
use crate::{TexasCard, TexasRuleSet};
use leocard_game_common::PlayerId;

impl TexasGameState {
    /// Start another hand with a fresh deck and move the dealer to the next funded seat.
    pub fn start_next_hand(&mut self, deck: Vec<TexasCard>) -> Result<(), TexasGameError> {
        if !matches!(self.phase, TexasPhase::Complete(_)) {
            return Err(TexasGameError::HandStillInProgress);
        }
        validate_deck(self.rules.short_deck, &deck)?;
        if self
            .players
            .iter()
            .filter(|player| player.stack > 0)
            .count()
            < 2
        {
            return Err(TexasGameError::NotEnoughFundedPlayers);
        }
        let dealer = self
            .next_funded(self.dealer)
            .expect("two funded players remain");
        self.hand_number = self.hand_number.saturating_add(1);
        self.begin_hand(dealer, deck)
    }

    pub fn table_winner(&self) -> Option<PlayerId> {
        let funded = self
            .players
            .iter()
            .filter(|player| player.stack > 0)
            .map(|player| player.id)
            .collect::<Vec<_>>();
        (funded.len() == 1).then_some(funded[0])
    }

    pub(super) fn begin_hand(
        &mut self,
        dealer: PlayerId,
        deck: Vec<TexasCard>,
    ) -> Result<(), TexasGameError> {
        self.squid.start_hand();
        self.dealer = dealer;
        self.deck = deck.into();
        self.community.clear();
        self.betting_history.clear();
        self.statistics.start_hand(&self.players);
        self.current_bet = 0;
        self.minimum_raise = TexasRuleSet::BIG_BLIND;
        self.full_bets = 0;
        self.all_in_lock = None;
        self.runout_voters.clear();
        self.runout_votes.clear();
        self.paid.fill(0);
        self.phase = TexasPhase::Betting(TexasStreet::PreFlop);
        self.pending_blind = Some(TexasBlindKind::Small);
        for player in &mut self.players {
            player.hole_cards.clear();
            player.folded = player.stack == 0;
            player.all_in = player.stack == 0;
            player.committed_street = 0;
            player.committed_total = 0;
        }
        let funded = self
            .players
            .iter()
            .filter(|player| player.stack > 0)
            .count();
        if funded < 2 {
            return Err(TexasGameError::NotEnoughFundedPlayers);
        }
        if self.players[dealer.index()].stack == 0 {
            return Err(TexasGameError::InvalidDealer(dealer));
        }
        self.small_blind = if funded == 2 {
            dealer
        } else {
            self.next_funded(dealer).expect("another funded player")
        };
        self.big_blind = self
            .next_funded(self.small_blind)
            .expect("another funded player");
        let first_dealt = self.next_funded(dealer).expect("another funded player");
        let mut dealt_to = first_dealt;
        for _ in 0..self.rules.hole_card_count() {
            loop {
                let card = self
                    .deck
                    .pop_front()
                    .expect("validated deck is large enough");
                self.players[dealt_to.index()].hole_cards.push(card);
                dealt_to = self.next_funded(dealt_to).expect("funded player ring");
                if dealt_to == first_dealt {
                    break;
                }
            }
        }
        self.needs_action.fill(false);
        self.raise_allowed.fill(false);
        self.current_player = Some(self.small_blind);
        Ok(())
    }

    pub(super) fn commit(&mut self, player: PlayerId, amount: u32) {
        let state = &mut self.players[player.index()];
        debug_assert!(amount <= state.stack);
        state.stack -= amount;
        state.committed_street += amount;
        state.committed_total += amount;
        self.paid[player.index()] += amount;
        state.all_in = state.stack == 0;
        if state.all_in {
            self.needs_action[player.index()] = false;
            if !self.rules.side_pots && self.all_in_lock.is_none() {
                self.all_in_lock = Some(state.committed_street);
            }
        }
    }

    pub(super) fn finish_betting_round(&mut self) -> Result<(), TexasGameError> {
        if self.all_in_lock.is_some() {
            self.refund_no_side_pot_excess();
            return self.close_betting();
        }
        if self.actionable_players().len() <= 1 {
            return self.close_betting();
        }
        let next_street = match self.street() {
            TexasStreet::PreFlop => TexasStreet::Flop,
            TexasStreet::Flop => TexasStreet::Turn,
            TexasStreet::Turn => TexasStreet::River,
            TexasStreet::River => return self.settle_showdown(None).map(|_| ()),
        };
        self.deal_community_for(next_street);
        self.phase = TexasPhase::Betting(next_street);
        self.current_bet = 0;
        self.minimum_raise = if self.rules.fixed_limit
            && matches!(next_street, TexasStreet::Turn | TexasStreet::River)
        {
            TexasRuleSet::BIG_BLIND * 2
        } else {
            TexasRuleSet::BIG_BLIND
        };
        self.full_bets = 0;
        self.needs_action.fill(false);
        self.raise_allowed.fill(false);
        for player in &mut self.players {
            player.committed_street = 0;
        }
        for index in 0..self.players.len() {
            if self.can_act(PlayerId::from_index(index)) {
                self.needs_action[index] = true;
                self.raise_allowed[index] = true;
            }
        }
        self.current_player = self.next_needing_action(self.dealer);
        Ok(())
    }

    pub(super) fn draw_runout(&mut self) {
        while self.community.len() < 5 {
            let street = match self.community.len() {
                0 => TexasStreet::Flop,
                3 => TexasStreet::Turn,
                4 => TexasStreet::River,
                _ => unreachable!("community is dealt as 0, 3, 4, 5"),
            };
            self.deal_community_for(street);
        }
    }

    fn deal_community_for(&mut self, street: TexasStreet) {
        let count = if street == TexasStreet::Flop { 3 } else { 1 };
        for _ in 0..count {
            self.community.push(
                self.deck
                    .pop_front()
                    .expect("validated deck is large enough"),
            );
        }
    }

    pub(super) fn street(&self) -> TexasStreet {
        match self.phase {
            TexasPhase::Betting(street) => street,
            TexasPhase::ChoosingRunout | TexasPhase::Complete(_) => TexasStreet::River,
        }
    }

    pub(super) fn contenders(&self) -> Vec<PlayerId> {
        self.players
            .iter()
            .filter(|player| !player.folded)
            .map(|player| player.id)
            .collect()
    }

    fn actionable_players(&self) -> Vec<PlayerId> {
        self.players
            .iter()
            .filter(|player| !player.folded && !player.all_in)
            .map(|player| player.id)
            .collect()
    }

    pub(super) fn can_act(&self, player: PlayerId) -> bool {
        self.players
            .get(player.index())
            .is_some_and(|player| !player.folded && !player.all_in)
    }

    pub(super) fn next_needing_action(&self, after: PlayerId) -> Option<PlayerId> {
        (1..=self.players.len())
            .map(|offset| PlayerId::from_index((after.index() + offset) % self.players.len()))
            .find(|player| self.needs_action[player.index()] && self.can_act(*player))
    }

    fn next_funded(&self, after: PlayerId) -> Option<PlayerId> {
        (1..=self.players.len())
            .map(|offset| PlayerId::from_index((after.index() + offset) % self.players.len()))
            .find(|player| self.players[player.index()].stack > 0)
    }

    pub(super) fn clockwise_distance(&self, from: PlayerId, to: PlayerId) -> usize {
        let distance = (to.index() + self.players.len() - from.index()) % self.players.len();
        if distance == 0 {
            self.players.len()
        } else {
            distance
        }
    }
}
