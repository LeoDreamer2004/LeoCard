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

    pub(super) fn score(&self, previous: Option<u32>, cycle: usize) -> GuessScore {
        let Self::Known {
            matches, correct, ..
        } = self
        else {
            return GuessScore {
                during_guess: 0,
                after_answer: 0,
            };
        };
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
        GuessScore {
            during_guess: u32::from(improved) + matching,
            after_answer: if *correct {
                5 + match cycle {
                    0 => 2,
                    1 => 1,
                    _ => 0,
                }
            } else {
                0
            },
        }
    }
}

/// The outer ranks receive the largest awards; extra middle places share zero.
/// Ties use the highest shared rank, as in the other games.
pub fn reference_point_deltas(scores: &[u32], rounds: u8) -> Option<Vec<i16>> {
    let tiers: &[i16] = match scores.len() {
        2 => &[2, -2],
        3 => &[2, 0, -2],
        4 => &[2, 1, -1, -2],
        5 => &[2, 1, 0, -1, -2],
        6 => &[2, 1, 0, 0, -1, -2],
        _ => return None,
    };
    let multiplier = if rounds >= 3 { 2 } else { 1 };
    let tiers: Vec<_> = tiers.iter().map(|tier| tier * multiplier).collect();
    ranked_awards(scores, &tiers)
}
