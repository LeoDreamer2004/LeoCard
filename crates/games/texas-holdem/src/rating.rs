use leocard_game_common::ranked_awards;

/// Reference awards for completed 3–6 player Texas Hold'em matches.
/// Ties take the highest tied rank, preserving the input player order.
pub fn reference_point_deltas(scores: &[u32]) -> Option<Vec<i16>> {
    let awards: &[i16] = match scores.len() {
        3 => &[3, 0, -3],
        4 => &[4, 2, -2, -4],
        5 => &[4, 2, 0, -2, -4],
        6 => &[6, 4, 2, -2, -4, -6],
        _ => return None,
    };
    ranked_awards(scores, awards)
}
