/// 按最终牌局得分从高到低计算2–6人局的参考积分变化。
///
/// 并列玩家都取得该并列区间中较高名次的档位。例如四人局的第二、三名
/// 同分时，两人都得到第二名的 `+1`。
pub fn reference_point_deltas(scores: &[u32]) -> Option<Vec<i16>> {
    let tiers: &[i16] = match scores.len() {
        2 => &[2, -2],
        3 => &[2, 0, -2],
        4 => &[3, 1, -1, -3],
        5 => &[3, 1, 0, -1, -3],
        6 => &[5, 3, 1, -1, -3, -5],
        _ => return None,
    };

    let mut ranked = scores.iter().copied().enumerate().collect::<Vec<_>>();
    ranked.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));

    let mut deltas = vec![0; scores.len()];
    let mut rank = 0;
    while rank < ranked.len() {
        let tied_score = ranked[rank].1;
        let mut end = rank + 1;
        while end < ranked.len() && ranked[end].1 == tied_score {
            end += 1;
        }
        for &(original_index, _) in &ranked[rank..end] {
            deltas[original_index] = tiers[rank];
        }
        rank = end;
    }
    Some(deltas)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distinct_scores_follow_every_configured_tier() {
        assert_eq!(reference_point_deltas(&[20, 10]), Some(vec![2, -2]));
        assert_eq!(reference_point_deltas(&[30, 20, 10]), Some(vec![2, 0, -2]));
        assert_eq!(
            reference_point_deltas(&[40, 30, 20, 10]),
            Some(vec![3, 1, -1, -3])
        );
        assert_eq!(
            reference_point_deltas(&[50, 40, 30, 20, 10]),
            Some(vec![3, 1, 0, -1, -3])
        );
        assert_eq!(
            reference_point_deltas(&[60, 50, 40, 30, 20, 10]),
            Some(vec![5, 3, 1, -1, -3, -5])
        );
    }

    #[test]
    fn ties_take_the_higher_tier_and_keep_original_player_order() {
        assert_eq!(
            reference_point_deltas(&[10, 30, 20, 20]),
            Some(vec![-3, 3, 1, 1])
        );
        assert_eq!(reference_point_deltas(&[20, 20]), Some(vec![2, 2]));
    }
}
