use super::GuessResult;
use crate::MatchQuality;
use leocard_game_common::ranked_awards;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ScoreGain {
    pub player: u8,
    pub before: u32,
    pub after: u32,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct GuessScore {
    pub during_guess: u32,
    pub after_answer: u32,
}

impl GuessResult {
    pub(super) fn closeness(&self) -> Option<u32> {
        let Self::Known { matches, .. } = self else {
            return None;
        };
        if matches.is_empty() {
            return None;
        }
        Some(
            matches
                .iter()
                .map(|feedback| match feedback.quality {
                    MatchQuality::Wrong => 0,
                    MatchQuality::Partial => 1,
                    MatchQuality::Exact => 2,
                })
                .sum(),
        )
    }

    pub(super) fn race_score(&self, first_matching_available: bool) -> GuessScore {
        GuessScore {
            during_guess: 3 * u32::from(
                !self.is_correct() && first_matching_available && self.all_matching(),
            ),
            after_answer: 5 * u32::from(self.is_correct()),
        }
    }

    pub(super) fn score(
        &self,
        previous: Option<u32>,
        cycle: usize,
        first_matching_available: bool,
    ) -> GuessScore {
        let Self::Known {
            matches, correct, ..
        } = self
        else {
            return GuessScore {
                during_guess: 0,
                after_answer: 0,
            };
        };
        if *correct {
            return GuessScore {
                during_guess: 0,
                after_answer: 5 + match cycle {
                    0 => 2,
                    1 => 1,
                    _ => 0,
                },
            };
        }
        if matches.is_empty() {
            return GuessScore {
                during_guess: 0,
                after_answer: 0,
            };
        }
        let improved = previous.is_none_or(|previous| self.closeness().unwrap() > previous);
        let differences = matches
            .iter()
            .filter(|feedback| feedback.quality != MatchQuality::Exact)
            .count();
        let matching = match differences {
            0 => 2,
            1 => 1,
            _ => 0,
        };
        let first_matching = 3 * u32::from(first_matching_available && self.all_matching());
        GuessScore {
            during_guess: u32::from(improved) + matching + first_matching,
            after_answer: 0,
        }
    }
}

/// Tied scores use the highest shared rank; long matches double every award.
pub fn reference_point_deltas(scores: &[u32], rounds: u8) -> Option<Vec<i16>> {
    let tiers: &[i16] = match scores.len() {
        2 => &[2, -2],
        3 => &[2, 0, -2],
        4 => &[3, -2, -2, -3],
        5 => &[3, 2, 0, -2, -3],
        6 => &[5, 3, 1, -1, -3, -5],
        7 => &[5, 3, 1, 0, -1, -3, -5],
        8 => &[5, 3, 2, 1, -1, -2, -3, -5],
        _ => return None,
    };
    let multiplier = if rounds >= 3 { 2 } else { 1 };
    let tiers: Vec<_> = tiers.iter().map(|tier| tier * multiplier).collect();
    ranked_awards(scores, &tiers)
}
