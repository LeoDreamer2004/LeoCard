use super::super::hand::{counts, face};
use super::evaluation::BurialProblem;
use crate::{ShengjiCard, ShengjiRuleSet, ShengjiTrump};

#[derive(Clone)]
pub(super) struct Candidate {
    pub cards: Vec<ShengjiCard>,
    pub value: f64,
}

impl BurialProblem<'_> {
    pub fn candidate(&self, mut cards: Vec<ShengjiCard>) -> Candidate {
        cards.sort_unstable_by_key(|card| (face(*card), card.deck()));
        let value = self.score(&cards);
        Candidate { cards, value }
    }

    /// Bounded whole-hand plans, also used before any optional timed search.
    pub fn plans(&self) -> Vec<Candidate> {
        let mut candidates = Vec::new();
        let copies = counts(self.hand);
        for protect_masters in [false, true] {
            for priority in 0..3 {
                let mut sorted = self.hand.to_vec();
                sorted.sort_by_key(|card| {
                    (
                        self.trump.is_trump(*card),
                        protect_masters && self.master(*card),
                        priority == 2 && copies[face(*card)] >= 2,
                        if priority == 1 {
                            -i32::from(card.points())
                        } else {
                            0
                        },
                        self.trump.strength(*card),
                        face(*card),
                        card.deck(),
                    )
                });
                candidates.push(self.candidate(sorted[..self.count].to_vec()));
                for mask in 1_u8..16 {
                    // Either void these suits completely, or keep only their masters.
                    let mut cards: Vec<_> = sorted
                        .iter()
                        .copied()
                        .filter(|card| {
                            !self.trump.is_trump(*card)
                                && card
                                    .suit()
                                    .is_some_and(|suit| mask & (1 << suit as u8) != 0)
                                && !(protect_masters && self.master(*card))
                        })
                        .collect();
                    if cards.len() > self.count {
                        continue;
                    }
                    for &card in &sorted {
                        if cards.len() == self.count {
                            break;
                        }
                        // Protected winners must not sneak back in while filling a plan.
                        let protected = protect_masters
                            && self.master(card)
                            && card
                                .suit()
                                .is_some_and(|suit| mask & (1 << suit as u8) != 0);
                        if !cards.contains(&card) && !protected {
                            cards.push(card);
                        }
                    }
                    if cards.len() == self.count {
                        candidates.push(self.candidate(cards));
                    }
                }
            }
        }
        rank(&mut candidates);
        candidates
    }
}

pub(super) fn rank(candidates: &mut Vec<Candidate>) {
    candidates.sort_by(|a, b| {
        b.value.total_cmp(&a.value).then_with(|| {
            a.cards
                .iter()
                .map(|card| (face(*card), card.deck()))
                .cmp(b.cards.iter().map(|card| (face(*card), card.deck())))
        })
    });
    candidates.dedup_by(|a, b| a.cards == b.cards);
}

pub(in crate::bot) fn quick_bury(
    hand: &[ShengjiCard],
    count: usize,
    trump: ShengjiTrump,
    rules: ShengjiRuleSet,
    attacking: bool,
) -> Vec<ShengjiCard> {
    BurialProblem::new(hand, count, trump, rules, attacking)
        .plans()
        .remove(0)
        .cards
}
