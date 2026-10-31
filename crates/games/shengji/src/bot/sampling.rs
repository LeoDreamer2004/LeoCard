use super::ShengjiBotObservation;
use super::knowledge::PublicKnowledge;
use super::observation::category_index;
use crate::play::category;
use crate::{ShengjiCard, ShengjiTrickPlay, classify_lead, validate_follow};
use leocard_game_common::{SearchBudget, SearchRandom};

#[derive(Clone)]
pub(super) struct SampledDeal {
    pub hands: Vec<Vec<ShengjiCard>>,
    pub bottom: Vec<ShengjiCard>,
}

/// Constrained assignments plus rejection against the authoritative follow
/// rules. Feasible samples are not claimed to be a uniform posterior.
pub(super) struct DealSampler<'a> {
    observation: &'a ShengjiBotObservation,
    slots: Vec<usize>,
    allowed: Vec<Vec<usize>>,
}

impl<'a> DealSampler<'a> {
    pub fn new(observation: &'a ShengjiBotObservation, public: &PublicKnowledge) -> Self {
        let bottom_seat = usize::from(observation.rules.player_count());
        let mut slots = Vec::new();
        for (seat, &count) in observation.lengths.iter().enumerate() {
            if seat != usize::from(observation.player.0) {
                slots.extend([seat].repeat(count.saturating_sub(observation.exposed[seat].len())));
            }
        }
        if observation.bottom.is_none() {
            slots.extend([bottom_seat].repeat(observation.rules.kitty_size()));
        }
        let allowed = observation
            .unseen
            .iter()
            .map(|card| {
                slots
                    .iter()
                    .enumerate()
                    .filter(|(_, seat)| {
                        **seat == bottom_seat
                            || !public.voids[**seat]
                                [category_index(category(*card, observation.trump))]
                    })
                    .map(|(index, _)| index)
                    .collect()
            })
            .collect();
        Self {
            observation,
            slots,
            allowed,
        }
    }

    pub fn sample(&self, random: &mut SearchRandom, budget: &SearchBudget) -> Option<SampledDeal> {
        if self.slots.len() != self.observation.unseen.len() {
            return None;
        }
        // Rejection against past tricks can be rare late in a hand. A failed
        // batch is not evidence of impossibility; use the caller's time budget.
        while !budget.exhausted() {
            let mut neighbors = self.allowed.clone();
            for options in &mut neighbors {
                shuffle(options, random);
            }
            let mut order: Vec<_> = (0..neighbors.len()).collect();
            shuffle(&mut order, random);
            order.sort_by_key(|index| neighbors[*index].len());
            let mut assigned = vec![None; self.slots.len()];
            let mut complete = true;
            for card in order {
                if !assign(
                    card,
                    &neighbors,
                    &mut assigned,
                    &mut vec![false; self.slots.len()],
                ) {
                    complete = false;
                    break;
                }
            }
            if !complete {
                continue;
            }
            let mut hands = self.observation.exposed.clone();
            hands[usize::from(self.observation.player.0)] = self.observation.hand.clone();
            let mut bottom = self.observation.bottom.clone().unwrap_or_default();
            for (slot, card) in assigned.into_iter().enumerate() {
                let card = self.observation.unseen[card.expect("complete matching")];
                if self.slots[slot] == hands.len() {
                    bottom.push(card);
                } else {
                    hands[self.slots[slot]].push(card);
                }
            }
            if self.consistent_history(&hands, budget) {
                return Some(SampledDeal { hands, bottom });
            }
        }
        None
    }

    fn consistent_history(&self, hands: &[Vec<ShengjiCard>], budget: &SearchBudget) -> bool {
        let mut before = hands.to_vec();
        for trick in self
            .observation
            .history
            .iter()
            .chain(self.observation.trick.iter())
            .rev()
        {
            if budget.exhausted() {
                return false;
            }
            for (seat, play) in &trick.plays {
                before[usize::from(seat.0)].extend_from_slice(&play.cards);
            }
            let (leader, lead) = &trick.plays[0];
            if lead.is_throw() {
                let others: Vec<_> = before
                    .iter()
                    .enumerate()
                    .filter(|(seat, _)| *seat != usize::from(leader.0))
                    .map(|(_, hand)| hand.as_slice())
                    .collect();
                if !matches!(
                    classify_lead(
                        &lead.cards,
                        self.observation.trump,
                        &self.observation.rules,
                        &others
                    ),
                    Ok(ShengjiTrickPlay::Accepted(_))
                ) {
                    return false;
                }
            }
            for (seat, play) in trick.plays.iter().skip(1) {
                if validate_follow(
                    &before[usize::from(seat.0)],
                    &play.cards,
                    &trick.plays[0].1,
                    self.observation.trump,
                )
                .is_err()
                {
                    return false;
                }
            }
        }
        true
    }
}

fn assign(
    card: usize,
    neighbors: &[Vec<usize>],
    assigned: &mut [Option<usize>],
    seen: &mut [bool],
) -> bool {
    for &slot in &neighbors[card] {
        if seen[slot] {
            continue;
        }
        seen[slot] = true;
        if assigned[slot].is_none_or(|other| assign(other, neighbors, assigned, seen)) {
            assigned[slot] = Some(card);
            return true;
        }
    }
    false
}

fn shuffle<T>(values: &mut [T], random: &mut SearchRandom) {
    for index in (1..values.len()).rev() {
        values.swap(index, random.index(index + 1));
    }
}
