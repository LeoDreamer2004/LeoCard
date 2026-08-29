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
    let placements = placements(winner, hand_scores);
    reference_point_deltas_for_placements(&placements)
}

pub(crate) fn placements(winner: PlayerId, hand_scores: &[u16]) -> Vec<u8> {
    placements_with_eliminations(winner, hand_scores, &[])
        .expect("winner and score list were validated")
}

pub(crate) fn placements_with_eliminations(
    winner: PlayerId,
    hand_scores: &[u16],
    elimination_order: &[PlayerId],
) -> Option<Vec<u8>> {
    let player_count = hand_scores.len();
    if !(2..=6).contains(&player_count) || winner.0 >= player_count {
        return None;
    }
    let mut eliminated = vec![false; player_count];
    for &player in elimination_order {
        if player.0 >= player_count || player == winner || eliminated[player.0] {
            return None;
        }
        eliminated[player.0] = true;
    }
    let mut placements = vec![0; hand_scores.len()];
    placements[winner.0] = 1;
    let mut ranked = hand_scores
        .iter()
        .copied()
        .enumerate()
        .filter(|(index, _)| *index != winner.0 && !eliminated[*index])
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
    for (index, player) in elimination_order.iter().copied().enumerate() {
        placements[player.0] = u8::try_from(player_count - index).ok()?;
    }
    Some(placements)
}

pub(crate) fn reference_point_deltas_for_placements(placements: &[u8]) -> Option<Vec<i16>> {
    if !(2..=6).contains(&placements.len()) {
        return None;
    }
    let tiers = TIERS[placements.len() - 2];
    placements
        .iter()
        .copied()
        .map(|placement| {
            placement
                .checked_sub(1)
                .and_then(|index| tiers.get(usize::from(index)))
                .copied()
        })
        .collect()
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

    #[test]
    fn eliminated_players_take_bottom_places_in_elimination_order() {
        let placements = placements_with_eliminations(
            PlayerId(0),
            &[0, 99, 5, 88, 20, 10],
            &[PlayerId(1), PlayerId(3)],
        )
        .unwrap();
        assert_eq!(placements, vec![1, 6, 2, 5, 4, 3]);
        assert_eq!(
            reference_point_deltas_for_placements(&placements),
            Some(vec![5, -3, 1, -2, -1, 0])
        );
    }
}
