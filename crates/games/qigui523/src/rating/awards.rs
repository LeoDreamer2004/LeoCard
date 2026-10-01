use leocard_game_common::ranked_awards;

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

    ranked_awards(scores, tiers)
}
