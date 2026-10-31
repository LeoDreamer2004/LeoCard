use super::ShengjiBotObservation;
use super::candidates::units;
use super::hand::{FACES, category_index, counts, face};
use crate::play::{category, classify_cards};
use crate::{
    ShengjiCard, ShengjiCategory, ShengjiClassifiedPlay, ShengjiComponent, ShengjiRank,
    ShengjiRuleSet, ShengjiSuit, ShengjiTeamId, ShengjiTrickPlay, ShengjiTrump, build_deck_for,
    classify_lead, compare_for_trick,
};
use leocard_game_common::PlayerId;

#[derive(Clone)]
pub(super) struct PublicKnowledge {
    pub remaining: [u8; FACES],
    pub voids: Vec<[bool; 5]>,
    pub multiplicity: Vec<[u8; 5]>,
}

impl PublicKnowledge {
    pub fn from_observation(observation: &ShengjiBotObservation) -> Self {
        let mut public = Self {
            remaining: counts(&build_deck_for(observation.rules.deck_count)),
            voids: vec![[false; 5]; usize::from(observation.rules.player_count())],
            multiplicity: vec![
                [observation.rules.deck_count; 5];
                usize::from(observation.rules.player_count())
            ],
        };
        for trick in observation.history.iter().chain(observation.trick.iter()) {
            for (player, play) in &trick.plays {
                public.record(*player, play, &trick.plays[0].1, observation.trump);
            }
        }
        public
    }

    pub fn record(
        &mut self,
        player: PlayerId,
        play: &ShengjiClassifiedPlay,
        lead: &ShengjiClassifiedPlay,
        trump: ShengjiTrump,
    ) {
        for &card in &play.cards {
            self.remaining[face(card)] = self.remaining[face(card)].saturating_sub(1);
        }
        let cat = super::observation::category_index(lead.category);
        if play
            .cards
            .iter()
            .any(|card| category(*card, trump) != lead.category)
        {
            self.voids[usize::from(player.0)][cat] = true;
        } else {
            let required = match lead.components.as_slice() {
                [ShengjiComponent::Pair { .. }] => 2,
                [ShengjiComponent::Triple { .. }] => 3,
                [ShengjiComponent::Quad { .. }] => 4,
                _ => return,
            };
            let shown = counts(&play.cards).into_iter().max().unwrap_or(0);
            if shown < required {
                self.multiplicity[usize::from(player.0)][cat] =
                    self.multiplicity[usize::from(player.0)][cat].min(shown);
            }
        }
    }
}

/// A seat's beliefs, rebuilt from public facts and that seat's private cards.
/// Probabilities are bounded heuristics; they are not calibrated win rates.
pub(super) struct Knowledge<'a> {
    pub player: PlayerId,
    pub trump: ShengjiTrump,
    pub public: &'a PublicKnowledge,
    pub lengths: &'a [usize],
    outside: [u8; FACES],
    shapes: Vec<ShengjiClassifiedPlay>,
}

impl<'a> Knowledge<'a> {
    pub fn new(
        player: PlayerId,
        hand: &[ShengjiCard],
        trump: ShengjiTrump,
        public: &'a PublicKnowledge,
        lengths: &'a [usize],
        bottom: Option<&[ShengjiCard]>,
    ) -> Self {
        let mut outside = public.remaining;
        for &card in hand.iter().chain(bottom.into_iter().flatten()) {
            outside[face(card)] = outside[face(card)].saturating_sub(1);
        }
        let pool = cards_from_counts(&outside);
        Self {
            player,
            trump,
            public,
            lengths,
            outside,
            shapes: units(&pool, trump),
        }
    }

    pub fn outside_count(&self, cat: usize) -> usize {
        self.outside
            .iter()
            .enumerate()
            .filter(|(index, _)| category_index(face_card(*index), self.trump) == cat)
            .map(|(_, count)| usize::from(*count))
            .sum()
    }

    pub fn outside_points(&self) -> f64 {
        self.outside
            .iter()
            .enumerate()
            .map(|(index, count)| f64::from(*count) * f64::from(face_card(index).points()))
            .sum()
    }

    pub fn master(&self, play: &ShengjiClassifiedPlay) -> bool {
        !self.shapes.iter().any(|other| {
            other.category == play.category
                && other.cards.len() == play.cards.len()
                && compare_for_trick(play, play, other, self.trump).is_gt()
        })
    }

    pub fn safe_throw(
        &self,
        cards: &[ShengjiCard],
        rules: ShengjiRuleSet,
    ) -> Option<ShengjiClassifiedPlay> {
        let pool = cards_from_counts(&self.outside);
        match classify_lead(cards, self.trump, &rules, &[&pool]).ok()? {
            ShengjiTrickPlay::Accepted(play) if play.is_throw() => Some(play),
            _ => None,
        }
    }

    pub fn void_probability(&self, seat: PlayerId, cat: usize) -> f64 {
        if self.public.voids[usize::from(seat.0)][cat] {
            return 1.0;
        }
        let allowed = self.allowed_count(seat);
        let same = self.outside_count(cat);
        let draws = self.lengths[usize::from(seat.0)].min(allowed);
        let mut probability = 1.0;
        for index in 0..draws {
            if allowed.saturating_sub(same) <= index {
                return 0.0;
            }
            probability *= (allowed - same - index) as f64 / (allowed - index) as f64;
        }
        probability
    }

    fn allowed_count(&self, seat: PlayerId) -> usize {
        self.outside
            .iter()
            .enumerate()
            .filter(|(index, _)| {
                !self.public.voids[usize::from(seat.0)]
                    [category_index(face_card(*index), self.trump)]
            })
            .map(|(_, count)| usize::from(*count))
            .sum()
    }

    fn holding_probability(&self, seat: PlayerId, play: &ShengjiClassifiedPlay) -> f64 {
        let cat = super::observation::category_index(play.category);
        if self.public.voids[usize::from(seat.0)][cat] {
            return 0.0;
        }
        let total = self.allowed_count(seat);
        let draws = self.lengths[usize::from(seat.0)].min(total);
        if draws < play.cards.len() {
            return 0.0;
        }
        let mut probability = 1.0;
        for (index, required) in counts(&play.cards)
            .into_iter()
            .enumerate()
            .filter(|(_, count)| *count > 0)
        {
            if required > self.public.multiplicity[usize::from(seat.0)][cat] {
                return 0.0;
            }
            probability *= copies_probability(total, draws, self.outside[index], required);
        }
        probability.clamp(0.0, 1.0)
    }

    pub fn threat(
        &self,
        seat: PlayerId,
        lead: &ShengjiClassifiedPlay,
        winner: &ShengjiClassifiedPlay,
    ) -> f64 {
        let ruff = if lead.category == ShengjiCategory::Trump {
            0.0
        } else {
            self.void_probability(seat, super::observation::category_index(lead.category))
        };
        let mut safe = 1.0;
        for shape in &self.shapes {
            if shape.cards.len() != lead.cards.len()
                || (shape.category != lead.category && shape.category != ShengjiCategory::Trump)
            {
                continue;
            }
            if compare_for_trick(lead, winner, shape, self.trump).is_gt() {
                let probability = self.holding_probability(seat, shape)
                    * if shape.category == lead.category {
                        1.0
                    } else {
                        ruff
                    };
                safe *= 1.0 - probability;
            }
        }
        // Composite throws require a matching collection of units. Treat their
        // strongest component as a conservative threat screen, never certainty.
        if lead.is_throw() {
            let component = classify_cards(&winner.strongest_component().cards(), self.trump)
                .expect("component");
            let mut component_safe = 1.0;
            for shape in &self.shapes {
                if shape.cards.len() == component.cards.len()
                    && (shape.category == component.category
                        || shape.category == ShengjiCategory::Trump)
                    && compare_for_trick(&component, &component, shape, self.trump).is_gt()
                {
                    let probability = self.holding_probability(seat, shape)
                        * if shape.category == lead.category {
                            1.0
                        } else {
                            ruff
                        };
                    component_safe *= 1.0 - probability;
                }
            }
            safe = safe.min(component_safe);
        }
        1.0 - safe
    }

    pub fn team_probability(
        &self,
        lead: &ShengjiClassifiedPlay,
        winner: &ShengjiClassifiedPlay,
        winner_seat: PlayerId,
        remaining: &[PlayerId],
    ) -> f64 {
        let mut probability =
            f64::from(ShengjiTeamId::of(winner_seat) == ShengjiTeamId::of(self.player));
        for &seat in remaining {
            let threat = self.threat(seat, lead, winner);
            if ShengjiTeamId::of(seat) == ShengjiTeamId::of(self.player) {
                probability += (1.0 - probability) * threat;
            } else {
                probability *= 1.0 - threat;
            }
        }
        probability
    }
}

pub(super) fn face_card(index: usize) -> ShengjiCard {
    match index {
        52 => ShengjiCard::small_joker(0),
        53 => ShengjiCard::big_joker(0),
        _ => ShengjiCard::suited(
            0,
            ShengjiSuit::ALL[index / 13],
            ShengjiRank::LEVELS[index % 13],
        ),
    }
}

fn cards_from_counts(counts: &[u8; FACES]) -> Vec<ShengjiCard> {
    counts
        .iter()
        .enumerate()
        .flat_map(|(index, count)| {
            (0..*count).map(move |deck| match index {
                52 => ShengjiCard::small_joker(deck),
                53 => ShengjiCard::big_joker(deck),
                _ => ShengjiCard::suited(
                    deck,
                    ShengjiSuit::ALL[index / 13],
                    ShengjiRank::LEVELS[index % 13],
                ),
            })
        })
        .collect()
}

fn copies_probability(total: usize, draws: usize, available: u8, required: u8) -> f64 {
    if required > available || total == 0 {
        return 0.0;
    }
    let mut probability = 0.0;
    for selected in required..=available {
        let mut term = 1.0;
        for index in 0..usize::from(available) {
            let numerator = if index < usize::from(selected) {
                draws.checked_sub(index)
            } else {
                (total - draws).checked_sub(index - usize::from(selected))
            };
            let Some(numerator) = numerator else {
                term = 0.0;
                break;
            };
            term *= numerator as f64 / (total - index) as f64;
        }
        let mut combinations = 1.0;
        for index in 0..selected {
            combinations *= f64::from(available - index) / f64::from(index + 1);
        }
        probability += combinations * term;
    }
    probability.clamp(0.0, 1.0)
}
