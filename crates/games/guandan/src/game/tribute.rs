use super::state::TributeExchange;
use super::{
    GuandanActionOutcome, GuandanGameError, GuandanGameResult, GuandanGameState, GuandanPhase,
    GuandanPlayerState,
};
use crate::{GuandanCard, GuandanRuleSet, build_deck};
use leocard_game_common::{DeckError, PlayerId, validate_deck};
use std::array;

impl GuandanGameState {
    pub fn new(
        rules: GuandanRuleSet,
        deck: Vec<GuandanCard>,
        starter: PlayerId,
    ) -> Result<Self, GuandanGameError> {
        Self::new_with_tribute(rules, deck, starter, None)
    }

    pub fn new_with_tribute(
        rules: GuandanRuleSet,
        deck: Vec<GuandanCard>,
        starter: PlayerId,
        previous: Option<GuandanGameResult>,
    ) -> Result<Self, GuandanGameError> {
        if !rules.is_valid() {
            return Err(GuandanGameError::InvalidRules);
        }
        if starter.index() >= 4 {
            return Err(GuandanGameError::InvalidPlayer(starter));
        }
        validate_deck(&deck, &build_deck()).map_err(|error| match error {
            DeckError::InvalidSize { expected, actual } => {
                GuandanGameError::InvalidDeckSize { expected, actual }
            }
            DeckError::InvalidContents => GuandanGameError::InvalidDeckContents,
        })?;
        let mut players: [GuandanPlayerState; 4] = array::from_fn(|index| GuandanPlayerState {
            id: PlayerId::from_index(index),
            hand: Vec::with_capacity(27),
        });
        for (index, card) in deck.into_iter().enumerate() {
            players[index % 4].hand.push(card);
        }
        for player in &mut players {
            player
                .hand
                .sort_unstable_by_key(|card| (card.rank, card.suit, card.deck));
        }
        let mut tribute_pairs = Vec::new();
        if let Some(result) = previous {
            if let Some(invalid) = result
                .finishing_order
                .iter()
                .find(|player| player.index() >= 4)
            {
                return Err(GuandanGameError::InvalidPlayer(*invalid));
            }
            let mut seats = result.finishing_order.map(|player| player.0);
            seats.sort_unstable();
            if seats != [0, 1, 2, 3] {
                return Err(GuandanGameError::InvalidRules);
            }
            let partner_position = result
                .finishing_order
                .iter()
                .position(|player| {
                    player.0 % 2 == result.finishing_order[0].0 % 2
                        && *player != result.finishing_order[0]
                })
                .expect("four distinct seats include the winner's partner");
            if result.winning_team != result.finishing_order[0].0 % 2
                || result.level_gain != (4 - partner_position) as u8
                || (result.known_positions != 2 && result.known_positions != 4)
                || (result.known_positions == 2 && result.level_gain != 3)
            {
                return Err(GuandanGameError::InvalidRules);
            }
            let order = result.finishing_order;
            if result.level_gain == 3 {
                let mut givers = [order[2], order[3]];
                givers.sort_unstable_by_key(|player| {
                    std::cmp::Reverse(highest_strength(
                        &players[player.index()].hand,
                        rules.starting_level,
                    ))
                });
                tribute_pairs.push((givers[0], order[0]));
                tribute_pairs.push((givers[1], order[1]));
            } else {
                tribute_pairs.push((order[3], order[0]));
            }
        }
        let tribute_resisted = tribute_pairs.iter().any(|(giver, _)| {
            players[giver.index()]
                .hand
                .iter()
                .filter(|card| card.is_big_joker())
                .count()
                == 2
        });
        let exchanges: Vec<_> = if tribute_resisted {
            Vec::new()
        } else {
            tribute_pairs
                .iter()
                .map(|(giver, receiver)| TributeExchange {
                    giver: *giver,
                    receiver: *receiver,
                    offered: None,
                    returned: false,
                })
                .collect()
        };
        let (phase, current_player) = if let Some(first) = exchanges.first() {
            (GuandanPhase::Tribute, first.giver)
        } else {
            (GuandanPhase::Playing, starter)
        };
        Ok(Self {
            rules,
            players,
            phase,
            starter,
            current_player,
            winning_player: None,
            winning_play: None,
            passes: 0,
            finishing_order: Vec::new(),
            played_cards: Vec::new(),
            tribute_pairs,
            exchanges,
            tribute_resisted,
            last_actions: array::from_fn(|_| None),
        })
    }

    pub(super) fn submit_tribute(
        &mut self,
        player: PlayerId,
        card: GuandanCard,
    ) -> Result<GuandanActionOutcome, GuandanGameError> {
        let Some(index) = self
            .exchanges
            .iter()
            .position(|exchange| !exchange.returned)
        else {
            return Err(GuandanGameError::WrongPhase);
        };
        if self.exchanges[index].offered.is_some() {
            return Err(GuandanGameError::WrongPhase);
        }
        let giver = self.exchanges[index].giver;
        let receiver = self.exchanges[index].receiver;
        if player != giver {
            return Err(GuandanGameError::NotPlayersTurn {
                expected: giver,
                actual: player,
            });
        }
        self.ensure_card_in_hand(player, card)?;
        if crate::play::rank_strength(card.rank, self.rules.starting_level)
            != highest_strength(&self.players[giver.index()].hand, self.rules.starting_level)
        {
            return Err(GuandanGameError::NotHighestTribute);
        }
        self.transfer_card(giver, receiver, card);
        self.exchanges[index].offered = Some(card);
        self.current_player = receiver;
        Ok(GuandanActionOutcome::TributeSubmitted {
            player,
            next_player: receiver,
        })
    }

    pub(super) fn return_tribute(
        &mut self,
        player: PlayerId,
        card: GuandanCard,
    ) -> Result<GuandanActionOutcome, GuandanGameError> {
        let Some(index) = self
            .exchanges
            .iter()
            .position(|exchange| !exchange.returned)
        else {
            return Err(GuandanGameError::WrongPhase);
        };
        let exchange = &self.exchanges[index];
        let Some(offered) = exchange.offered else {
            return Err(GuandanGameError::WrongPhase);
        };
        let giver = exchange.giver;
        let receiver = exchange.receiver;
        if player != receiver {
            return Err(GuandanGameError::NotPlayersTurn {
                expected: receiver,
                actual: player,
            });
        }
        self.ensure_card_in_hand(player, card)?;
        let eligible = self.players[receiver.index()]
            .hand
            .iter()
            .filter(|candidate| **candidate != offered)
            .any(|candidate| candidate.rank <= 10);
        if card == offered
            || (eligible && card.rank > 10)
            || (!eligible
                && crate::play::rank_strength(card.rank, self.rules.starting_level)
                    != lowest_return_strength(
                        &self.players[receiver.index()].hand,
                        offered,
                        self.rules.starting_level,
                    ))
        {
            return Err(GuandanGameError::InvalidReturnTribute);
        }
        self.transfer_card(receiver, giver, card);
        self.exchanges[index].returned = true;
        let next = if let Some(exchange) = self.exchanges.iter().find(|exchange| !exchange.returned)
        {
            exchange.giver
        } else {
            self.phase = GuandanPhase::Playing;
            self.starter
        };
        self.current_player = next;
        Ok(GuandanActionOutcome::TributeReturned {
            player,
            next_player: next,
        })
    }

    pub(super) fn ensure_card_in_hand(
        &self,
        player: PlayerId,
        card: GuandanCard,
    ) -> Result<(), GuandanGameError> {
        if self.players[player.index()].hand.contains(&card) {
            Ok(())
        } else {
            Err(GuandanGameError::CardNotInHand(card))
        }
    }

    fn transfer_card(&mut self, giver: PlayerId, receiver: PlayerId, card: GuandanCard) {
        let hand = &mut self.players[giver.index()].hand;
        let position = hand
            .iter()
            .position(|candidate| *candidate == card)
            .expect("validated card");
        hand.remove(position);
        self.players[receiver.index()].hand.push(card);
        self.players[receiver.index()]
            .hand
            .sort_unstable_by_key(|card| (card.rank, card.suit, card.deck));
    }
}

pub(super) fn highest_strength(hand: &[GuandanCard], level: u8) -> u8 {
    hand.iter()
        .map(|card| crate::play::rank_strength(card.rank, level))
        .max()
        .unwrap_or(0)
}

fn lowest_return_strength(hand: &[GuandanCard], offered: GuandanCard, level: u8) -> u8 {
    hand.iter()
        .filter(|card| **card != offered)
        .map(|card| crate::play::rank_strength(card.rank, level))
        .min()
        .unwrap_or(0)
}
