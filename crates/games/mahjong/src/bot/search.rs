use super::MahjongBotObservation;
use super::shanten::{Counts, Shanten};
use super::targets::potential;
use crate::{
    MahjongMeld, MahjongMeldKind, MahjongScoreInput, MahjongTileKind, MahjongWinSource, score_hand,
};
use leocard_game_common::{PlayerId, SearchBudget};

use std::collections::HashMap;

#[derive(Clone)]
pub(super) struct Position {
    pub counts: Counts,
    pub melds: Vec<MahjongMeld>,
    pub replacement: bool,
}

#[derive(Hash, Eq, PartialEq)]
struct SearchKey {
    counts: Counts,
    unseen: Counts,
    melds: Vec<(u8, usize)>,
    replacement: bool,
    depth: u8,
}

pub(super) struct Search<'a> {
    pub observation: &'a MahjongBotObservation,
    pub budget: &'a SearchBudget,
    shanten: Shanten,
    values: HashMap<SearchKey, f64>,
}

impl<'a> Search<'a> {
    pub fn new(observation: &'a MahjongBotObservation, budget: &'a SearchBudget) -> Self {
        Self {
            observation,
            budget,
            shanten: Shanten::default(),
            values: HashMap::new(),
        }
    }

    pub fn distance(&mut self, position: &Position) -> i8 {
        self.shanten
            .calculate(&position.counts, position.melds.len() as u8)
    }

    pub fn evaluate(&mut self, position: &Position, unseen: &Counts) -> f64 {
        let distance = self.distance(position);
        let mut counts = position.counts;
        let mut width = 0.0;
        let mut legal_waits = 0.0;
        for index in 0..34 {
            if unseen[index] == 0 || counts[index] >= 4 {
                continue;
            }
            counts[index] += 1;
            let next = self.shanten.calculate(&counts, position.melds.len() as u8);
            if next < distance {
                let weight = if next < 0 {
                    if self.win_value(position, &counts, index).is_some() {
                        legal_waits += f64::from(unseen[index]);
                        1.0
                    } else {
                        0.15
                    }
                } else {
                    1.0
                };
                width += f64::from(unseen[index]) * weight;
            }
            counts[index] -= 1;
        }
        // Structural tenpai with no live legal win must not trap an 8-fan bot indefinitely.
        let effective_distance = if distance == 0 && legal_waits == 0.0 {
            1
        } else {
            distance.max(0)
        };
        -f64::from(effective_distance) * 90.0
            + width * 1.5
            + potential(&position.counts, &position.melds) * 6.0
    }

    pub fn best_discards(
        &mut self,
        position: &Position,
        unseen: &Counts,
        limit: usize,
    ) -> Vec<(usize, f64)> {
        let mut choices = Vec::new();
        let mut after = position.clone();
        for index in 0..34 {
            if after.counts[index] == 0 {
                continue;
            }
            after.counts[index] -= 1;
            choices.push((
                index,
                self.evaluate(&after, unseen) - self.observation.danger(index),
            ));
            after.counts[index] += 1;
        }
        choices.sort_by(|left, right| right.1.total_cmp(&left.1));
        choices.truncate(limit);
        choices
    }

    /// Weighted, finite-horizon own-draw search. Unseen tiles are not assumed to be in the wall.
    pub fn future(&mut self, position: &Position, unseen: &Counts, depth: u8) -> Option<f64> {
        if self.budget.exhausted() {
            return None;
        }
        let base = self.evaluate(position, unseen);
        let distance = self.distance(position);
        if depth == 0 || distance >= 3 {
            return Some(base);
        }
        let key = SearchKey {
            counts: position.counts,
            unseen: *unseen,
            melds: position
                .melds
                .iter()
                .map(|meld| {
                    (
                        match meld.kind() {
                            MahjongMeldKind::Chow => 0,
                            MahjongMeldKind::Pung => 1,
                            MahjongMeldKind::Kong(_) if meld.is_open() => 2,
                            MahjongMeldKind::Kong(_) => 3,
                        },
                        meld.tile().index34().expect("meld tile"),
                    )
                })
                .collect(),
            replacement: position.replacement,
            depth,
        };
        if let Some(value) = self.values.get(&key) {
            return Some(*value);
        }
        let total = unseen.iter().map(|count| f64::from(*count)).sum::<f64>();
        if total == 0.0 {
            return Some(base);
        }
        let mut value = base;
        let mut drawn = position.clone();
        let mut remaining = *unseen;
        for index in 0..34 {
            if self.budget.exhausted() {
                return None;
            }
            if remaining[index] == 0 || drawn.counts[index] >= 4 {
                continue;
            }
            let weight = f64::from(remaining[index]) / total;
            drawn.counts[index] += 1;
            remaining[index] -= 1;
            let next_distance = self.distance(&drawn);
            let win = (next_distance < 0)
                .then(|| self.win_value(position, &drawn.counts, index))
                .flatten();
            let next_value = if let Some(win) = win {
                win
            } else if next_distance <= distance {
                let choices = self.best_discards(&drawn, &remaining, 2);
                let mut best = base;
                for (discard, _) in choices {
                    drawn.counts[discard] -= 1;
                    drawn.replacement = false;
                    let future = self.future(&drawn, &remaining, depth - 1)?
                        - self.observation.danger(discard);
                    best = best.max(future);
                    drawn.counts[discard] += 1;
                }
                best
            } else {
                base
            };
            drawn.counts[index] -= 1;
            remaining[index] += 1;
            drawn.replacement = position.replacement;
            value += weight * (next_value - base);
        }
        self.values.insert(key, value);
        Some(value)
    }

    fn win_value(&self, position: &Position, counts: &Counts, index: usize) -> Option<f64> {
        let tile = MahjongTileKind::from_index34(index)?;
        let concealed = counts
            .iter()
            .enumerate()
            .flat_map(|(index, count)| {
                (0..*count).map(move |_| MahjongTileKind::from_index34(index).expect("tile index"))
            })
            .collect();
        let mut context = self.observation.context;
        context.source = if position.replacement {
            MahjongWinSource::KongReplacement
        } else {
            MahjongWinSource::SelfDraw
        };
        context.last_wall_tile = self.observation.wall == 1;
        context.last_of_kind = self.observation.unseen[index] <= 1;
        let mut input = MahjongScoreInput {
            concealed,
            melds: position.melds.clone(),
            winning_tile: tile,
            context,
        };
        let self_draw = score_hand(&input).ok()?;
        let legal = |points| !self.observation.rules.minimum_eight_points || points >= 8;
        if !legal(self_draw.points_without_flowers) {
            return None;
        }
        input.context.source = MahjongWinSource::Discard(PlayerId::from_index(
            (self.observation.player.index() + 1) % 4,
        ));
        let claim = score_hand(&input)
            .ok()
            .filter(|score| legal(score.points_without_flowers));
        Some(
            300.0
                + f64::from(self_draw.total_points) * 3.0
                + claim.map_or(0.0, |score| 25.0 + f64::from(score.total_points)),
        )
    }
}
