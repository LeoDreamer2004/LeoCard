use crate::app::runtime::DEEPSEEK_MARKER_ASSETS;
use leocard_protocol::MatchId;

/// Cosmetic draws use only public round identity, never viewer, owner or local time.
/// A fresh stream per marker keeps its choice independent of other marker draws.
pub(super) fn deepseek_marker(match_id: MatchId, round: u32, index: usize) -> Option<usize> {
    let left = u64::from_le_bytes(match_id.0[..8].try_into().expect("match ID half"));
    let right = u64::from_le_bytes(match_id.0[8..].try_into().expect("match ID half"));
    let seed = left
        ^ right.rotate_left(17)
        ^ u64::from(round).wrapping_mul(0x9e37_79b9_7f4a_7c15)
        ^ (index as u64).wrapping_mul(0xd1b5_4a32_d192_ed03);
    let mut random = fastrand::Rng::with_seed(seed);
    (random.u8(..6) == 0).then(|| usize::from(random.u8(..DEEPSEEK_MARKER_ASSETS.len() as u8)))
}

#[cfg(test)]
mod tests {
    use super::deepseek_marker;
    use crate::app::runtime::DEEPSEEK_MARKER_ASSETS;
    use leocard_protocol::MatchId;

    #[test]
    fn public_round_identity_reproduces_markers_and_marker_count_does_not_affect_choices() {
        let id = MatchId([42; 16]);
        let first: Vec<_> = (0..5).map(|index| deepseek_marker(id, 1, index)).collect();
        assert_eq!(
            first,
            (0..5)
                .map(|index| deepseek_marker(id, 1, index))
                .collect::<Vec<_>>()
        );
        assert_eq!(
            &first[..3],
            &(0..3)
                .map(|index| deepseek_marker(id, 1, index))
                .collect::<Vec<_>>()
        );
        let draws = |id, round| {
            (0..64)
                .map(|index| deepseek_marker(id, round, index))
                .collect::<Vec<_>>()
        };
        assert_ne!(draws(id, 1), draws(id, 2));
        let mut changed = id;
        changed.0[15] ^= 1;
        assert_ne!(draws(id, 1), draws(changed, 1));
    }

    #[test]
    fn independent_draws_have_one_sixth_frequency_and_cover_all_eight_characters() {
        let id = MatchId([17; 16]);
        let mut variants = [0; DEEPSEEK_MARKER_ASSETS.len()];
        let mut replacements = 0;
        let mut pairs = 0;
        for round in 1..=9600 {
            let mut previous = false;
            for index in 0..5 {
                let choice = deepseek_marker(id, round, index);
                if let Some(image) = choice {
                    variants[image] += 1;
                    replacements += 1;
                }
                if index > 0 && previous && choice.is_some() {
                    pairs += 1;
                }
                previous = choice.is_some();
            }
        }
        assert!((7700..8300).contains(&replacements));
        assert!(
            variants
                .into_iter()
                .all(|count| (850..1150).contains(&count))
        );
        // Adjacent draws can both be replacements; the expected joint rate is 1/36.
        assert!((900..1250).contains(&pairs));
    }
}
