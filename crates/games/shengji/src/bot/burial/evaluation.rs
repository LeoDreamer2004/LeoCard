use super::super::hand::{category_index, counts, face, hand_value};
use crate::{ShengjiCard, ShengjiRuleSet, ShengjiTrump, build_deck_for};

pub(super) struct BurialProblem<'a> {
    pub hand: &'a [ShengjiCard],
    pub count: usize,
    pub trump: ShengjiTrump,
    pub attacking: bool,
    outside_strength: [Option<u8>; 5],
    trump_total: usize,
    players: u8,
}

impl<'a> BurialProblem<'a> {
    pub fn new(
        hand: &'a [ShengjiCard],
        count: usize,
        trump: ShengjiTrump,
        rules: ShengjiRuleSet,
        attacking: bool,
    ) -> Self {
        let deck = build_deck_for(rules.deck_count);
        let trump_total = deck.iter().filter(|card| trump.is_trump(**card)).count();
        let mut outside_strength: [Option<u8>; 5] = [None; 5];
        for card in deck.into_iter().filter(|card| !hand.contains(card)) {
            let maximum = &mut outside_strength[category_index(card, trump)];
            *maximum = Some(maximum.unwrap_or(0).max(trump.strength(card)));
        }
        Self {
            hand,
            count,
            trump,
            attacking,
            outside_strength,
            trump_total,
            players: rules.player_count(),
        }
    }

    pub fn kept(&self, buried: &[ShengjiCard]) -> Vec<ShengjiCard> {
        self.hand
            .iter()
            .copied()
            .filter(|card| !buried.contains(card))
            .collect()
    }

    /// A master cannot be beaten in its own category given our private hand.
    /// This is not a guarantee against ruffs or losing the lead.
    pub fn master(&self, card: ShengjiCard) -> bool {
        self.outside_strength[category_index(card, self.trump)]
            .is_none_or(|strength| self.trump.strength(card) >= strength)
    }

    pub fn score(&self, buried: &[ShengjiCard]) -> f64 {
        let kept = self.kept(buried);
        let copies = counts(&kept);
        let trumps: Vec<_> = kept
            .iter()
            .copied()
            .filter(|card| self.trump.is_trump(*card))
            .collect();
        let average = (self.trump_total - trumps.len()) as f64 / f64::from(self.players - 1);
        let high = trumps
            .iter()
            .filter(|card| card.suit().is_none() || card.rank() == self.trump.level)
            .count();
        let pairs = trumps
            .iter()
            .filter(|card| copies[face(**card)] >= 2)
            .count();
        // Conservative last-trick control estimate, never certainty merely from length.
        let control = (0.15
            + (trumps.len() as f64 / average.max(1.0) - 1.0) * 0.28
            + high as f64 * 0.04
            + pairs as f64 * 0.018)
            .clamp(0.08, 0.8);
        let bottom_points: f64 = buried.iter().map(|card| f64::from(card.points())).sum();
        let bottom_value = if self.attacking {
            // Points hidden by an attacking copier are useful only if our team
            // can capture the bottom. Otherwise they remove scoring opportunities.
            bottom_points * (control * 3.0 - 1.0) * 0.7
        } else {
            -bottom_points * (1.0 - control) * 2.4 * 0.7
        };
        let mut value = hand_value(&kept, self.trump) + bottom_value;
        for category in 0..4 {
            let suit: Vec<_> = kept
                .iter()
                .copied()
                .filter(|card| category_index(*card, self.trump) == category)
                .collect();
            // Do not reward an already void suit, or a void without ruffing material.
            let original = self
                .hand
                .iter()
                .filter(|card| category_index(**card, self.trump) == category)
                .count();
            if suit.is_empty() && original > 0 && !trumps.is_empty() {
                value += 5.0 + control * 13.0;
                continue;
            }
            let masters = suit.iter().filter(|card| self.master(**card)).count();
            value += masters as f64 * 3.5;
            if !suit.is_empty() && suit.len() == masters && !trumps.is_empty() {
                // Cash a winner, then ruff: near-void plans are valuable as a whole.
                value += 3.0 + control * 7.0;
            }
            for card in &suit {
                if !self.master(*card) {
                    let protection = (masters as f64 * 0.25
                        + if copies[face(*card)] >= 2 { 0.2 } else { 0.0 })
                    .min(0.7);
                    value -= f64::from(card.points()) * (1.0 - protection) * 0.85;
                }
            }
        }
        // Keep sure entries and top trump control for the final trick.
        value += trumps.iter().filter(|card| self.master(**card)).count() as f64 * 4.0;
        value
    }
}
