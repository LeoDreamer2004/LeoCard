use crate::PlayerId;

const TIERS: [&[i16]; 5] = [
    &[2, -2],
    &[3, -1, -2],
    &[3, 0, -1, -2],
    &[4, 1, 0, -2, -3],
    &[5, 1, 0, -1, -2, -3],
];

/// 胜者固定第一，其余玩家按剩余手牌分数从低到高排名。
///
/// 并列者都取得并列区间中较高名次的档位，后续名次按并列人数跳过。
pub fn reference_point_deltas(winner: PlayerId, hand_scores: &[u16]) -> Option<Vec<i16>> {
    if !(2..=6).contains(&hand_scores.len()) || winner.0 >= hand_scores.len() {
        return None;
    }
    let tiers = TIERS[hand_scores.len() - 2];
    let mut deltas = vec![0; hand_scores.len()];
    deltas[winner.0] = tiers[0];

    let mut ranked = hand_scores
        .iter()
        .copied()
        .enumerate()
        .filter(|(index, _)| *index != winner.0)
        .collect::<Vec<_>>();
    ranked.sort_by(|left, right| left.1.cmp(&right.1).then_with(|| left.0.cmp(&right.0)));

    let mut rank = 0;
    while rank < ranked.len() {
        let tied_score = ranked[rank].1;
        let mut end = rank + 1;
        while end < ranked.len() && ranked[end].1 == tied_score {
            end += 1;
        }
        for &(original_index, _) in &ranked[rank..end] {
            deltas[original_index] = tiers[rank + 1];
        }
        rank = end;
    }
    Some(deltas)
}

pub(crate) fn placements(winner: PlayerId, hand_scores: &[u16]) -> Vec<u8> {
    let mut placements = vec![0; hand_scores.len()];
    placements[winner.0] = 1;
    let mut ranked = hand_scores
        .iter()
        .copied()
        .enumerate()
        .filter(|(index, _)| *index != winner.0)
        .collect::<Vec<_>>();
    ranked.sort_by(|left, right| left.1.cmp(&right.1).then_with(|| left.0.cmp(&right.0)));
    let mut rank = 0;
    while rank < ranked.len() {
        let tied_score = ranked[rank].1;
        let mut end = rank + 1;
        while end < ranked.len() && ranked[end].1 == tied_score {
            end += 1;
        }
        for &(index, _) in &ranked[rank..end] {
            placements[index] = (rank + 2) as u8;
        }
        rank = end;
    }
    placements
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn configured_tiers_cover_two_through_six_players() {
        assert_eq!(
            reference_point_deltas(PlayerId(0), &[0, 10]),
            Some(vec![2, -2])
        );
        assert_eq!(
            reference_point_deltas(PlayerId(1), &[10, 0, 20]),
            Some(vec![-1, 3, -2])
        );
        assert_eq!(
            reference_point_deltas(PlayerId(0), &[0, 10, 20, 30]),
            Some(vec![3, 0, -1, -2])
        );
        assert_eq!(
            reference_point_deltas(PlayerId(0), &[0, 10, 20, 30, 40]),
            Some(vec![4, 1, 0, -2, -3])
        );
        assert_eq!(
            reference_point_deltas(PlayerId(0), &[0, 10, 20, 30, 40, 50]),
            Some(vec![5, 1, 0, -1, -2, -3])
        );
    }

    #[test]
    fn tied_losers_take_the_higher_tier_and_skip_following_places() {
        assert_eq!(
            reference_point_deltas(PlayerId(0), &[0, 10, 10, 30]),
            Some(vec![3, 0, 0, -2])
        );
        assert_eq!(placements(PlayerId(0), &[0, 10, 10, 30]), vec![1, 2, 2, 4]);
    }
}
