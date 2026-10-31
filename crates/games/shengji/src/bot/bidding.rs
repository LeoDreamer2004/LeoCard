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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build_deck;

    fn make_hand(hearts: usize, spades: usize) -> Vec<ShengjiCard> {
        let mut hand = vec![
            ShengjiCard::suited(0, ShengjiSuit::Heart, ShengjiRank::Two),
            ShengjiCard::suited(1, ShengjiSuit::Heart, ShengjiRank::Two),
        ];
        let deck = build_deck();
        for (suit, count) in [
            (ShengjiSuit::Heart, hearts - 2),
            (ShengjiSuit::Spade, spades),
        ] {
            hand.extend(
                deck.iter()
                    .copied()
                    .filter(|card| card.suit() == Some(suit) && card.rank() != ShengjiRank::Two)
                    .take(count),
            );
        }
        let missing = 25 - hand.len();
        hand.extend(
            deck.into_iter()
                .filter(|card| {
                    card.suit() == Some(ShengjiSuit::Diamond) && card.rank() != ShengjiRank::Two
                })
                .take(missing),
        );
        hand
    }

    fn observation<'a>(
        bidding: &'a ShengjiBidState,
        hand: &'a [ShengjiCard],
    ) -> ShengjiBiddingObservation<'a> {
        ShengjiBiddingObservation {
            bidding,
            player: PlayerId(0),
            hand,
            rules: ShengjiRuleSet::default(),
            dealer: Some(PlayerId(0)),
            dealt: 100,
        }
    }

    #[test]
    fn opening_bids_immediately_and_protects_with_only_new_cards() {
        let mut bidding = ShengjiBidState::new(ShengjiRank::Two);
        let hand = make_hand(8, 10);
        let first = hand[0];
        let mut request = observation(&bidding, &hand[..1]);
        request.dealer = None;
        request.dealt = 1;
        assert_eq!(ShengjiBot::declaration(&request), Some(vec![first]));
        bidding.declare(PlayerId(0), &[first], &hand).unwrap();
        let mut request = observation(&bidding, &hand);
        request.dealer = None;
        let cards = ShengjiBot::declaration(&request).unwrap();
        assert_eq!(cards, vec![hand[1]]);
        assert_eq!(
            bidding
                .declare(request.player, &cards, &hand)
                .unwrap()
                .cards
                .len(),
            2
        );
    }

    #[test]
    fn no_trump_can_also_be_strengthened_without_self_countering_to_another_suit() {
        let mut bidding = ShengjiBidState::new_with_decks(ShengjiRank::Two, 4);
        let hand: Vec<_> = (0..4).map(ShengjiCard::big_joker).collect();
        bidding.declare(PlayerId(1), &hand[..2], &hand).unwrap();
        let mut request = observation(&bidding, &hand);
        request.player = PlayerId(1);
        request.rules.deck_count = 4;
        request.dealt = 208;
        assert_eq!(ShengjiBot::declaration(&request), Some(hand[2..].to_vec()));
    }

    #[test]
    fn dealer_and_partner_wait_and_respect_counter_thresholds() {
        let mut bidding = ShengjiBidState::new(ShengjiRank::Two);
        let hand = make_hand(8, 10);
        let mut request = observation(&bidding, &hand);
        request.dealt = 49;
        assert!(ShengjiBot::declaration(&request).is_none());
        request.dealt = 50;
        assert_eq!(ShengjiBot::declaration(&request).unwrap().len(), 2);
        request.player = PlayerId(2);
        request.dealt = 99;
        assert!(ShengjiBot::declaration(&request).is_none());
        request.dealt = 100;
        assert!(ShengjiBot::declaration(&request).is_some());
        let card = ShengjiCard::suited(0, ShengjiSuit::Spade, ShengjiRank::Two);
        bidding.declare(PlayerId(1), &[card], &[card]).unwrap();
        for (player, hearts, expected) in [(0, 7, false), (0, 8, true), (2, 6, false), (2, 7, true)]
        {
            let hand = make_hand(hearts, 10);
            let mut request = observation(&bidding, &hand);
            request.player = PlayerId(player);
            assert_eq!(ShengjiBot::declaration(&request).is_some(), expected);
        }
        let mut dealer_bid = ShengjiBidState::new(ShengjiRank::Two);
        dealer_bid.declare(PlayerId(0), &[card], &[card]).unwrap();
        let mut request = observation(&dealer_bid, &hand);
        request.player = PlayerId(2);
        assert!(ShengjiBot::declaration(&request).is_none());
    }

    #[test]
    fn opponents_observe_all_four_rule_combinations() {
        let bidding = ShengjiBidState::new(ShengjiRank::Two);
        for (power, copy, dealt, hearts, expected) in [
            (true, true, 100, 12, false),
            (false, true, 99, 12, false),
            (false, true, 100, 9, false),
            (false, true, 100, 10, true),
            (true, false, 100, 12, false),
            (false, false, 49, 12, false),
            (false, false, 50, 7, false),
            (false, false, 50, 8, true),
        ] {
            let hand = make_hand(hearts, 0);
            let mut request = observation(&bidding, &hand);
            request.player = PlayerId(1);
            request.rules.power_outage_dealer = power;
            request.rules.bottom_copy = copy;
            request.dealt = dealt;
            assert_eq!(
                ShengjiBot::declaration(&request).is_some(),
                expected,
                "{power}/{copy}/{dealt}/{hearts}"
            );
        }
        let mut bidding = bidding;
        let card = ShengjiCard::suited(0, ShengjiSuit::Spade, ShengjiRank::Two);
        bidding.declare(PlayerId(0), &[card], &[card]).unwrap();
        for (power, copy, dealt, hearts, expected) in [
            (true, true, 100, 12, false),
            (false, true, 100, 12, false),
            (true, false, 66, 8, false),
            (true, false, 67, 7, false),
            (true, false, 67, 8, true),
            (false, false, 49, 9, false),
            (false, false, 50, 8, false),
            (false, false, 50, 9, true),
        ] {
            let hand = make_hand(hearts, 10);
            let mut request = observation(&bidding, &hand);
            request.player = PlayerId(1);
            request.rules.power_outage_dealer = power;
            request.rules.bottom_copy = copy;
            request.dealt = dealt;
            assert_eq!(
                ShengjiBot::declaration(&request).is_some(),
                expected,
                "{power}/{copy}/{dealt}/{hearts}"
            );
        }
    }

    #[test]
    fn longest_suit_is_chosen_and_own_bid_is_only_strengthened_without_copy() {
        let mut bidding = ShengjiBidState::new(ShengjiRank::Two);
        let mut hand = make_hand(8, 9);
        hand.push(ShengjiCard::suited(0, ShengjiSuit::Spade, ShengjiRank::Two));
        hand.push(ShengjiCard::suited(1, ShengjiSuit::Spade, ShengjiRank::Two));
        let request = observation(&bidding, &hand);
        let cards = ShengjiBot::declaration(&request).unwrap();
        assert!(
            cards
                .iter()
                .all(|card| card.suit() == Some(ShengjiSuit::Spade))
        );
        let first = hand[0];
        bidding.declare(PlayerId(1), &[first], &hand).unwrap();
        let mut request = observation(&bidding, &hand);
        request.player = PlayerId(1);
        assert_eq!(ShengjiBot::declaration(&request), Some(vec![hand[1]]));
    }
}
