use super::{GameError, GameState, HandState, HandStatus, Phase, PlayerState, PlayerStatistics};
use crate::{BlackjackCard, BlackjackRuleSet, HandValue, build_shoe};
use std::collections::HashMap;

impl GameState {
    pub fn new(
        rules: BlackjackRuleSet,
        player_count: usize,
        shoe: Vec<BlackjackCard>,
    ) -> Result<Self, GameError> {
        if !rules.is_valid() {
            return Err(GameError::InvalidRules);
        }
        if !(usize::from(BlackjackRuleSet::MIN_PLAYERS)
            ..=usize::from(BlackjackRuleSet::MAX_PLAYERS))
            .contains(&player_count)
        {
            return Err(GameError::InvalidPlayerCount);
        }
        validate_shoe(&shoe)?;
        let mut game = Self {
            rules,
            players: (0..player_count)
                .map(|_| PlayerState {
                    stack: rules.initial_chips,
                    round_start_stack: rules.initial_chips,
                    hands: Vec::new(),
                    statistics: PlayerStatistics::default(),
                })
                .collect(),
            dealer: Vec::new(),
            shoe,
            phase: Phase::Betting,
            round: 1,
            current: Some(0),
            hand: 0,
        };
        game.shoe.reverse();
        Ok(game)
    }

    pub fn start_next_round(&mut self, mut shoe: Vec<BlackjackCard>) -> Result<(), GameError> {
        if self.phase != Phase::Complete {
            return Err(GameError::WrongPhase);
        }
        if self.match_complete() {
            return Err(GameError::MatchComplete);
        }
        validate_shoe(&shoe)?;
        shoe.reverse();
        self.shoe = shoe;
        self.round += 1;
        self.dealer.clear();
        self.phase = Phase::Betting;
        self.hand = 0;
        for player in &mut self.players {
            player.hands.clear();
            player.round_start_stack = player.stack;
        }
        self.current = self
            .players
            .iter()
            .position(|player| player.stack >= self.rules.minimum_bet);
        Ok(())
    }

    pub(super) fn bet(&mut self, player: usize, amount: u32) {
        self.players[player].stack -= amount;
        self.players[player].hands.push(HandState {
            cards: Vec::new(),
            wager: amount,
            status: HandStatus::Playing,
            split: false,
            doubled: false,
            result: None,
            net_chips: 0,
        });
        self.current = self.players.iter().enumerate().find_map(|(index, player)| {
            (player.stack >= self.rules.minimum_bet && player.hands.is_empty()).then_some(index)
        });
        if self.current.is_none() {
            self.deal();
        }
    }

    fn deal(&mut self) {
        for _ in 0..2 {
            for index in 0..self.players.len() {
                if !self.players[index].hands.is_empty() {
                    let card = self.draw();
                    self.players[index].hands[0].cards.push(card);
                }
            }
            let card = self.draw();
            self.dealer.push(card);
        }
        for player in &mut self.players {
            if let Some(hand) = player.hands.first_mut()
                && hand.value().total == 21
            {
                hand.status = HandStatus::Blackjack;
            }
        }
        self.phase = Phase::Playing;
        if HandValue::of(&self.dealer).total == 21 {
            self.settle();
        } else {
            self.advance_play();
        }
    }

    pub(super) fn advance_play(&mut self) {
        let next = self.players.iter().enumerate().find_map(|(player, state)| {
            state
                .hands
                .iter()
                .position(|hand| hand.status == HandStatus::Playing)
                .map(|hand| (player, hand))
        });
        if let Some((player, hand)) = next {
            self.current = Some(player);
            self.hand = hand;
            return;
        }
        let needs_comparison = self
            .players
            .iter()
            .flat_map(|player| &player.hands)
            .any(|hand| hand.status == HandStatus::Standing);
        if needs_comparison {
            while HandValue::of(&self.dealer).total < 17 {
                let card = self.draw();
                self.dealer.push(card);
            }
        }
        self.settle();
    }

    pub(super) fn draw(&mut self) -> BlackjackCard {
        // 完整六副牌的总点数远超过六人各四手及庄家所能消耗的点数。
        self.shoe
            .pop()
            .expect("a validated six-deck shoe cannot be exhausted in one round")
    }
}

fn validate_shoe(shoe: &[BlackjackCard]) -> Result<(), GameError> {
    let expected = build_shoe();
    if shoe.len() != expected.len() {
        return Err(GameError::InvalidShoe);
    }
    let mut counts = HashMap::new();
    for &card in shoe {
        *counts.entry(card).or_insert(0_u16) += 1;
    }
    if expected.iter().any(|card| counts.get(card) != Some(&6)) {
        return Err(GameError::InvalidShoe);
    }
    Ok(())
}
