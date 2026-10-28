use leocard_game_common::ranked_awards;

/// Reference awards for completed 3–8 player Texas Hold'em matches.
/// Ties take the highest tied rank, preserving the input player order.
/// Profitable players cannot lose points; losing players cannot gain points.
/// Breaking even always awards zero points.
pub fn reference_point_deltas(scores: &[u32], starting_chips: u16) -> Option<Vec<i16>> {
    let awards: &[i16] = match scores.len() {
        3 => &[3, 0, -3],
        4 => &[4, 2, -2, -4],
        5 => &[4, 2, 0, -2, -4],
        6 => &[6, 4, 2, -2, -4, -6],
        7 => &[6, 4, 2, 0, -2, -4, -6],
        8 => &[6, 4, 3, 2, -2, -3, -4, -6],
        _ => return None,
    };
    let mut deltas = ranked_awards(scores, awards)?;
    for (score, delta) in scores.iter().zip(&mut deltas) {
        if *score >= u32::from(starting_chips) {
            *delta = (*delta).max(0);
        }
        if *score <= u32::from(starting_chips) {
            *delta = (*delta).min(0);
        }
    }
    Some(deltas)
}

#[cfg(test)]
mod tests {
    use super::reference_point_deltas;

    #[test]
    fn final_chip_gains_and_losses_limit_rank_awards_in_player_order() {
        for (scores, expected) in [
            (vec![40, 20, 0], vec![3, 0, -3]),
            (vec![60, 15, 5, 0], vec![4, 0, -2, -4]),
            (vec![24, 0, 30, 26], vec![0, -4, 4, 2]),
            (vec![20, 20, 20, 20], vec![0, 0, 0, 0]),
            (
                vec![30, 29, 28, 27, 26, 20, 0, 0],
                vec![6, 4, 3, 2, 0, 0, -4, -4],
            ),
        ] {
            assert_eq!(reference_point_deltas(&scores, 20), Some(expected));
        }
    }

    #[test]
    fn compares_against_the_configured_starting_chips() {
        for starting_chips in [5, 20, 100] {
            assert_eq!(
                reference_point_deltas(&[u32::from(starting_chips) * 3, 0, 0], starting_chips),
                Some(vec![3, 0, 0])
            );
            assert_eq!(
                reference_point_deltas(&[u32::from(starting_chips); 3], starting_chips),
                Some(vec![0; 3])
            );
        }
        assert_eq!(reference_point_deltas(&[100, 100], 100), None);
        assert_eq!(reference_point_deltas(&[], 20), None);
    }
}
