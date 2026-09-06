use super::*;

impl GameState {
    /// 结束后使用一副新洗好的牌开始下一手；庄家按钮顺时针移动到下一名有筹码玩家。
    pub fn start_next_hand(&mut self, deck: Vec<TexasHoldemCard>) -> Result<(), GameError> {
        if !matches!(self.phase, Phase::Complete(_)) {
            return Err(GameError::HandStillInProgress);
        }
        validate_deck(self.rules.short_deck, &deck)?;
        if self
            .players
            .iter()
            .filter(|player| player.stack > 0)
            .count()
            < 2
        {
            return Err(GameError::NotEnoughFundedPlayers);
        }
        let dealer = self
            .next_funded(self.dealer)
            .expect("at least two funded players remain");
        self.hand_number = self.hand_number.saturating_add(1);
        self.begin_hand(dealer, deck)
    }

    pub fn table_winner(&self) -> Option<TexasHoldemPlayerId> {
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
        dealer: TexasHoldemPlayerId,
        deck: Vec<TexasHoldemCard>,
    ) -> Result<(), GameError> {
        self.dealer = dealer;
        self.deck = deck.into();
        self.community.clear();
        self.current_bet = 0;
        self.minimum_raise = TexasHoldemRuleSet::BIG_BLIND;
        self.phase = Phase::Betting(TexasHoldemStreet::PreFlop);
        self.pending_blind = Some(TexasHoldemBlindKind::Small);
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
            return Err(GameError::NotEnoughFundedPlayers);
        }
        if self.players[dealer.0].stack == 0 {
            return Err(GameError::InvalidDealer(dealer));
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
                self.players[dealt_to.0].hole_cards.push(card);
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

    pub(super) fn commit(&mut self, player: TexasHoldemPlayerId, amount: u32) {
        let state = &mut self.players[player.0];
        debug_assert!(amount <= state.stack);
        state.stack -= amount;
        state.committed_street += amount;
        state.committed_total += amount;
        state.all_in = state.stack == 0;
        if state.all_in {
            self.needs_action[player.0] = false;
        }
    }

    pub(super) fn finish_betting_round(&mut self) -> Result<(), GameError> {
        if self.actionable_players().len() <= 1 {
            return self.run_out_and_showdown();
        }
        let next_street = match self.street() {
            TexasHoldemStreet::PreFlop => TexasHoldemStreet::Flop,
            TexasHoldemStreet::Flop => TexasHoldemStreet::Turn,
            TexasHoldemStreet::Turn => TexasHoldemStreet::River,
            TexasHoldemStreet::River => return self.settle_showdown().map(|_| ()),
        };
        self.deal_community_for(next_street);
        self.phase = Phase::Betting(next_street);
        self.current_bet = 0;
        self.minimum_raise = TexasHoldemRuleSet::BIG_BLIND;
        self.needs_action.fill(false);
        self.raise_allowed.fill(false);
        for player in &mut self.players {
            player.committed_street = 0;
        }
        for index in 0..self.players.len() {
            if self.can_act(TexasHoldemPlayerId(index)) {
                self.needs_action[index] = true;
                self.raise_allowed[index] = true;
            }
        }
        self.current_player = self.next_needing_action(self.dealer);
        Ok(())
    }

    pub(super) fn run_out_and_showdown(&mut self) -> Result<(), GameError> {
        while self.community.len() < 5 {
            let street = match self.community.len() {
                0 => TexasHoldemStreet::Flop,
                3 => TexasHoldemStreet::Turn,
                4 => TexasHoldemStreet::River,
                _ => unreachable!("community is dealt as 0, 3, 4, 5"),
            };
            self.deal_community_for(street);
        }
        self.settle_showdown().map(|_| ())
    }

    fn deal_community_for(&mut self, street: TexasHoldemStreet) {
        let count = if street == TexasHoldemStreet::Flop {
            3
        } else {
            1
        };
        for _ in 0..count {
            self.community.push(
                self.deck
                    .pop_front()
                    .expect("validated deck is large enough"),
            );
        }
    }

    pub(super) fn settle_uncontested(&mut self) -> HandResult {
        let winner = self.contenders()[0];
        let amount = self.pot();
        self.players[winner.0].stack += amount;
        let result = HandResult {
            dealer: self.dealer,
            showdown: false,
            community: self.community.clone(),
            awards: vec![PotAward {
                amount,
                winners: vec![winner],
                winning_hand: None,
            }],
            final_stacks: self.players.iter().map(PlayerState::stack).collect(),
        };
        self.current_player = None;
        self.phase = Phase::Complete(result.clone());
        result
    }

    fn settle_showdown(&mut self) -> Result<HandResult, GameError> {
        let contenders = self.contenders();
        let mut evaluated = vec![None; self.players.len()];
        for player in &contenders {
            evaluated[player.0] = Some(evaluate_player_hand(
                &self.players[player.0].hole_cards,
                &self.community,
                &self.rules,
            )?);
        }

        let mut levels = self
            .players
            .iter()
            .map(PlayerState::committed_total)
            .filter(|amount| *amount > 0)
            .collect::<Vec<_>>();
        levels.sort_unstable();
        levels.dedup();
        let mut previous = 0;
        let mut awards = Vec::new();
        for level in levels {
            let participants = self
                .players
                .iter()
                .filter(|player| player.committed_total >= level)
                .count() as u32;
            let amount = (level - previous) * participants;
            previous = level;
            if amount == 0 {
                continue;
            }
            let eligible = contenders
                .iter()
                .copied()
                .filter(|player| self.players[player.0].committed_total >= level)
                .collect::<Vec<_>>();
            let best = eligible
                .iter()
                .filter_map(|player| evaluated[player.0])
                .max_by(|left, right| left.cmp_with_rules(right, &self.rules))
                .expect("each pot has an eligible contender");
            let mut winners = eligible
                .into_iter()
                .filter(|player| {
                    evaluated[player.0]
                        .is_some_and(|hand| hand.cmp_with_rules(&best, &self.rules).is_eq())
                })
                .collect::<Vec<_>>();
            winners.sort_by_key(|winner| {
                let distance = self.clockwise_distance(self.dealer, *winner);
                if distance == 0 {
                    self.players.len()
                } else {
                    distance
                }
            });
            let share = amount / winners.len() as u32;
            let remainder = amount % winners.len() as u32;
            for (index, winner) in winners.iter().enumerate() {
                self.players[winner.0].stack += share + u32::from((index as u32) < remainder);
            }
            awards.push(PotAward {
                amount,
                winners,
                winning_hand: Some(best),
            });
        }
        let result = HandResult {
            dealer: self.dealer,
            showdown: true,
            community: self.community.clone(),
            awards,
            final_stacks: self.players.iter().map(PlayerState::stack).collect(),
        };
        self.current_player = None;
        self.phase = Phase::Complete(result.clone());
        Ok(result)
    }

    pub(super) fn street(&self) -> TexasHoldemStreet {
        match self.phase {
            Phase::Betting(street) => street,
            Phase::Complete(_) => TexasHoldemStreet::River,
        }
    }

    pub(super) fn contenders(&self) -> Vec<TexasHoldemPlayerId> {
        self.players
            .iter()
            .filter(|player| !player.folded)
            .map(|player| player.id)
            .collect()
    }

    fn actionable_players(&self) -> Vec<TexasHoldemPlayerId> {
        self.players
            .iter()
            .filter(|player| !player.folded && !player.all_in)
            .map(|player| player.id)
            .collect()
    }

    pub(super) fn can_act(&self, player: TexasHoldemPlayerId) -> bool {
        self.players
            .get(player.0)
            .is_some_and(|player| !player.folded && !player.all_in)
    }

    pub(super) fn next_needing_action(
        &self,
        after: TexasHoldemPlayerId,
    ) -> Option<TexasHoldemPlayerId> {
        (1..=self.players.len())
            .map(|offset| TexasHoldemPlayerId((after.0 + offset) % self.players.len()))
            .find(|player| self.needs_action[player.0] && self.can_act(*player))
    }

    fn next_funded(&self, after: TexasHoldemPlayerId) -> Option<TexasHoldemPlayerId> {
        (1..=self.players.len())
            .map(|offset| TexasHoldemPlayerId((after.0 + offset) % self.players.len()))
            .find(|player| self.players[player.0].stack > 0)
    }

    fn clockwise_distance(&self, from: TexasHoldemPlayerId, to: TexasHoldemPlayerId) -> usize {
        (to.0 + self.players.len() - from.0) % self.players.len()
    }
}
