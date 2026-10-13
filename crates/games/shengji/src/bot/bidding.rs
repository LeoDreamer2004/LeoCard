use super::ShengjiBot;
use super::candidates::hand_value;
use crate::{
    BidState, ShengjiCard, ShengjiPlayerId, ShengjiRank, ShengjiRuleSet, ShengjiSuit, ShengjiTrump,
    bid_joker_for_suit,
};

impl ShengjiBot {
    pub fn declaration(
        bidding: &BidState,
        player: ShengjiPlayerId,
        hand: &[ShengjiCard],
        rules: ShengjiRuleSet,
        force: bool,
    ) -> Option<Vec<ShengjiCard>> {
        let mut candidates = Vec::new();
        for suit in ShengjiSuit::ALL {
            let levels: Vec<_> = hand
                .iter()
                .copied()
                .filter(|card| card.rank() == bidding.level() && card.suit() == Some(suit))
                .collect();
            for size in 1..=levels.len() {
                let mut cards = levels[..size].to_vec();
                if rules.bid_with_joker {
                    let Some(joker) = hand
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
            let cards: Vec<_> = hand
                .iter()
                .copied()
                .filter(|card| card.rank() == rank)
                .collect();
            if cards.len() >= 2 {
                candidates.push(cards);
            }
        }
        let mut best = None;
        let mut value = f64::NEG_INFINITY;
        for cards in candidates {
            let mut state = bidding.clone();
            let Ok(declaration) = state.declare(player, &cards, hand) else {
                continue;
            };
            let trump = ShengjiTrump::new(bidding.level(), declaration.trump.trump_suit())
                .ok()?
                .with_constant_trump(rules.constant_trump);
            let trumps = hand.iter().filter(|card| trump.is_trump(**card)).count();
            if !force
                && (hand.len() < 8 || trumps < 4 || (trumps as f64 / hand.len() as f64) < 0.27)
            {
                continue;
            }
            let strength = hand_value(hand, trump);
            if let Some(current) = bidding.current() {
                let old = ShengjiTrump::new(bidding.level(), current.trump.trump_suit())
                    .ok()?
                    .with_constant_trump(rules.constant_trump);
                let margin = if current.player.team() == player.team() {
                    7.0
                } else {
                    2.0
                };
                if !force && strength < hand_value(hand, old) + margin {
                    continue;
                }
            }
            if strength > value {
                value = strength;
                best = Some(cards);
            }
        }
        best
    }
}
