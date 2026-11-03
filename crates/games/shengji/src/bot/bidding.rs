use super::ShengjiBot;
use super::hand::trump_value;
use crate::ShengjiTeamId;
use crate::{
    ShengjiBidState, ShengjiCard, ShengjiDeclaration, ShengjiGameState, ShengjiPhase, ShengjiRank,
    ShengjiRuleSet, ShengjiSuit, ShengjiTrump, bid_joker_for_suit,
};
use leocard_game_common::PlayerId;

/// Only the bot's own hand and public bidding information are needed for declarations.
pub struct ShengjiBiddingObservation<'a> {
    bidding: &'a ShengjiBidState,
    player: PlayerId,
    hand: &'a [ShengjiCard],
    rules: ShengjiRuleSet,
    dealer: Option<PlayerId>,
    dealt: usize,
}

impl<'a> ShengjiBiddingObservation<'a> {
    pub fn from_game(game: &'a ShengjiGameState, player: PlayerId) -> Option<Self> {
        if !matches!(
            game.phase(),
            ShengjiPhase::Dealing | ShengjiPhase::BiddingGrace
        ) {
            return None;
        }
        Some(Self {
            bidding: game.bidding(),
            player,
            hand: &game.players().get(usize::from(player.0))?.hand,
            rules: *game.rules(),
            dealer: game.fixed_dealer(),
            dealt: game.players().iter().map(|player| player.hand.len()).sum(),
        })
    }

    fn suit_count(&self, suit: Option<ShengjiSuit>) -> usize {
        self.hand.iter().filter(|card| card.suit() == suit).count()
    }

    fn already_in_current_declaration(&self, card: &ShengjiCard) -> bool {
        self.bidding.current().is_some_and(|current| {
            current.player == self.player
                && current.trump.trump_suit() == card.suit()
                && current.cards.contains(card)
        })
    }

    fn permits(&self, declaration: &ShengjiDeclaration) -> bool {
        let Some(dealer) = self.dealer else {
            return true;
        };
        let total = self.rules.hand_size() * usize::from(self.rules.player_count());
        let complete = self.dealt >= total;
        let current = self.bidding.current();
        let protecting = current.is_some_and(|current| {
            current.player == self.player && current.trump == declaration.trump
        });
        let threshold_divisor = if declaration.trump.trump_suit().is_none() {
            3
        } else {
            1
        };
        let enough_suit = |percent: usize| {
            current.is_some_and(|current| {
                self.suit_count(declaration.trump.trump_suit()) * 100 * threshold_divisor
                    >= self.suit_count(current.trump.trump_suit()) * percent
            })
        };
        let enough_trumps = |percent: usize| {
            let trump = ShengjiTrump::new(self.bidding.level(), declaration.trump.trump_suit())
                .expect("valid bidding level")
                .with_constant_trump(self.rules.constant_trump);
            self.hand
                .iter()
                .filter(|card| trump.is_trump(**card))
                .count()
                * 100
                * threshold_divisor
                >= self.hand.len() * percent
        };
        if self.player == dealer {
            return self.dealt * 2 >= total && (current.is_none() || protecting || enough_suit(80));
        }
        if ShengjiTeamId::of(self.player) == ShengjiTeamId::of(dealer) {
            return complete
                && current.is_none_or(|current| current.player != dealer)
                && (current.is_none() || protecting || enough_suit(70));
        }
        match (self.rules.power_outage_dealer, self.rules.bottom_copy) {
            (true, true) => false,
            (false, true) => {
                complete
                    && (protecting
                        || (current.is_none_or(|current| current.player == self.player)
                            && enough_trumps(40)))
            }
            (true, false) => {
                self.dealt * 3 >= total * 2 && current.is_some() && (protecting || enough_suit(80))
            }
            (false, false) => {
                self.dealt * 2 >= total
                    && (protecting
                        || match current {
                            None => enough_trumps(30),
                            Some(current) => current.player != self.player && enough_suit(90),
                        })
            }
        }
    }
}

impl ShengjiBot {
    pub fn declaration(observation: &ShengjiBiddingObservation<'_>) -> Option<Vec<ShengjiCard>> {
        let mut candidates = Vec::new();
        for suit in ShengjiSuit::ALL {
            let levels: Vec<_> = observation
                .hand
                .iter()
                .copied()
                .filter(|card| {
                    card.rank() == observation.bidding.level()
                        && card.suit() == Some(suit)
                        && !observation.already_in_current_declaration(card)
                })
                .collect();
            for size in 1..=levels.len() {
                let mut cards = levels[..size].to_vec();
                if observation.rules.bid_with_joker {
                    let Some(joker) = observation
                        .hand
                        .iter()
                        .copied()
                        .find(|card| card.rank() == bid_joker_for_suit(suit))
                    else {
                        continue;
                    };
                    cards.push(joker);
                }
                candidates.push(cards);
            }
        }
        for rank in [ShengjiRank::SmallJoker, ShengjiRank::BigJoker] {
            let cards: Vec<_> = observation
                .hand
                .iter()
                .copied()
                .filter(|card| {
                    card.rank() == rank && !observation.already_in_current_declaration(card)
                })
                .collect();
            for size in 1..=cards.len() {
                candidates.push(cards[..size].to_vec());
            }
        }
        candidates
            .into_iter()
            .filter_map(|cards| {
                let mut bidding = observation.bidding.clone();
                let declaration = bidding
                    .declare(observation.player, &cards, observation.hand)
                    .ok()?;
                if !observation.permits(declaration) {
                    return None;
                }
                if let Some(current) = observation.bidding.current()
                    && current.player != observation.player
                    && ShengjiTeamId::of(current.player) == ShengjiTeamId::of(observation.player)
                {
                    let before =
                        ShengjiTrump::new(observation.bidding.level(), current.trump.trump_suit())
                            .expect("level")
                            .with_constant_trump(observation.rules.constant_trump);
                    let after = ShengjiTrump::new(
                        observation.bidding.level(),
                        declaration.trump.trump_suit(),
                    )
                    .expect("level")
                    .with_constant_trump(observation.rules.constant_trump);
                    if trump_value(observation.hand, after)
                        < trump_value(observation.hand, before) * 1.15 + 2.0
                    {
                        return None;
                    }
                }
                let suit = declaration.trump.trump_suit();
                let primary_count = declaration
                    .cards
                    .iter()
                    .filter(|card| card.suit() == suit)
                    .count();
                let strength = declaration
                    .trump
                    .declaration_strength(observation.rules.deck_count, primary_count)?;
                let quality = if observation.dealer.is_none() {
                    0.0
                } else {
                    let trump = ShengjiTrump::new(observation.bidding.level(), suit)
                        .expect("level")
                        .with_constant_trump(observation.rules.constant_trump);
                    trump_value(observation.hand, trump)
                };
                Some(((quality, strength), cards))
            })
            .max_by(|(left, _), (right, _)| left.0.total_cmp(&right.0).then(left.1.cmp(&right.1)))
            .map(|(_, cards)| cards)
    }
}
