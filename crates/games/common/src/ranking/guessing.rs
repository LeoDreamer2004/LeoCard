use super::ranked_awards;

/// Tied scores use the highest shared rank; long matches double every award.
pub fn guessing_reference_point_deltas(scores: &[u32], rounds: u8) -> Option<Vec<i16>> {
    let tiers: &[i16] = match scores.len() {
        2 => &[2, -2],
        3 => &[2, 0, -2],
        4 => &[3, 2, -2, -3],
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
