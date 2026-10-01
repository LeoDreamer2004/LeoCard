//! Shared mechanics without any game-specific award policy.

/// Apply rank awards in input order; tied scores take the highest tied rank.
pub fn ranked_awards<Score: Ord>(scores: &[Score], awards: &[i16]) -> Option<Vec<i16>> {
    if scores.len() != awards.len() || scores.is_empty() {
        return None;
    }
    Some(
        scores
            .iter()
            .map(|score| awards[scores.iter().filter(|other| *other > score).count()])
            .collect(),
    )
}
