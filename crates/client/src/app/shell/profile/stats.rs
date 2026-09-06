use super::*;

pub fn qigui523_profile_rows(stats: Option<&QiGui523ProfileStats>) -> Vec<(&'static str, String)> {
    const LABELS: [&str; 18] = [
        "对局数",
        "分数增减",
        "场得分",
        "平均顺位",
        "一位率",
        "二位率",
        "三位率",
        "四位率",
        "五位率",
        "六位率",
        "顺子次数",
        "连对次数",
        "飞机次数",
        "炸弹次数",
        "天炸次数",
        "顺子最长长度",
        "连对最长长度",
        "飞机最长长度",
    ];
    let Some(stats) = stats.filter(|stats| stats.completed_games > 0) else {
        return LABELS
            .into_iter()
            .map(|label| (label, "--".to_owned()))
            .collect();
    };
    let games = f64::from(stats.completed_games);
    let average_reference_delta = stats.total_reference_delta as f64 / games;
    let average_score = stats.total_score as f64 / games;
    let placement_total = stats
        .placement_counts
        .iter()
        .enumerate()
        .map(|(index, count)| (index + 1) as u64 * u64::from(*count))
        .sum::<u64>();
    let average_placement = placement_total as f64 / games;
    let mut rows = vec![
        ("对局数", stats.completed_games.to_string()),
        ("分数增减", format!("{average_reference_delta:+.1}")),
        ("场得分", format!("{average_score:.1}")),
        ("平均顺位", format!("{average_placement:.2}")),
    ];
    for (label, count) in LABELS[4..10].iter().copied().zip(stats.placement_counts) {
        rows.push((label, format!("{:.1}%", f64::from(count) * 100.0 / games)));
    }
    rows.extend([
        ("顺子次数", stats.straight_plays.to_string()),
        ("连对次数", stats.consecutive_pair_plays.to_string()),
        ("飞机次数", stats.airplane_plays.to_string()),
        ("炸弹次数", stats.bomb_plays.to_string()),
        ("天炸次数", stats.heaven_bomb_plays.to_string()),
        ("顺子最长长度", stats.longest_straight.to_string()),
        ("连对最长长度", stats.longest_consecutive_pairs.to_string()),
        ("飞机最长长度", stats.longest_airplane.to_string()),
    ]);
    rows
}
pub fn texas_holdem_profile_rows(
    stats: Option<&TexasHoldemProfileStats>,
) -> Vec<(&'static str, String)> {
    const LABELS: [&str; 25] = [
        "对局数",
        "分数增减",
        "场筹码",
        "平均顺位",
        "一位率",
        "二位率",
        "三位率",
        "四位率",
        "五位率",
        "六位率",
        "平均加注",
        "过牌率",
        "加注率",
        "全下率",
        "局弃牌率",
        "高牌次数",
        "一对次数",
        "两对次数",
        "三条次数",
        "顺子次数",
        "同花次数",
        "葫芦次数",
        "四条次数",
        "同花顺次数",
        "皇家同花顺次数",
    ];
    let Some(stats) = stats.filter(|stats| stats.completed_games > 0) else {
        return LABELS
            .into_iter()
            .map(|label| (label, "--".to_owned()))
            .collect();
    };
    let games = f64::from(stats.completed_games);
    let placement_total = stats
        .placement_counts
        .iter()
        .enumerate()
        .map(|(index, count)| (index + 1) as u64 * u64::from(*count))
        .sum::<u64>();
    let mut rows = vec![
        ("对局数", stats.completed_games.to_string()),
        (
            "分数增减",
            format!("{:+.1}", stats.total_reference_delta as f64 / games),
        ),
        (
            "场筹码",
            format!("{:.1}", stats.total_final_chips as f64 / games),
        ),
        ("平均顺位", format!("{:.2}", placement_total as f64 / games)),
    ];
    for (label, count) in LABELS[4..10].iter().copied().zip(stats.placement_counts) {
        rows.push((label, format!("{:.1}%", f64::from(count) * 100.0 / games)));
    }
    rows.extend([
        (
            "平均加注",
            average_or_placeholder(stats.wagered_chips, stats.wager_actions),
        ),
        (
            "过牌率",
            rate_or_placeholder(stats.check_actions, stats.voluntary_actions),
        ),
        (
            "加注率",
            rate_or_placeholder(stats.raise_actions, stats.voluntary_actions),
        ),
        (
            "全下率",
            rate_or_placeholder(stats.all_in_actions, stats.voluntary_actions),
        ),
        (
            "局弃牌率",
            rate_or_placeholder(stats.hands_folded, stats.hands_played),
        ),
    ]);
    rows.extend(
        LABELS[15..]
            .iter()
            .copied()
            .zip(stats.hand_category_counts)
            .map(|(label, count)| (label, count.to_string())),
    );
    rows
}

fn average_or_placeholder(total: u64, count: u32) -> String {
    if count == 0 {
        "--".to_owned()
    } else {
        format!("{:.1}", total as f64 / f64::from(count))
    }
}

fn rate_or_placeholder(count: u32, total: u32) -> String {
    if total == 0 {
        "--".to_owned()
    } else {
        format!(
            "{:.1}%",
            f64::from(count.min(total)) * 100.0 / f64::from(total)
        )
    }
}

pub fn shengji_profile_rows(stats: Option<&ShengjiProfileStats>) -> Vec<(&'static str, String)> {
    const LABELS: [&str; 21] = [
        "对局数",
        "分数增减",
        "庄场得分",
        "闲场得分",
        "坐庄率",
        "亮主率",
        "反主率",
        "保底率",
        "扣底率",
        "底牌平均分",
        "牌权率",
        "过江率",
        "拖拉机次数",
        "泰坦尼克次数",
        "炸弹次数",
        "太空堡垒次数",
        "甩牌次数",
        "最长拖拉机长度",
        "最长泰坦尼克长度",
        "最长太空堡垒长度",
        "最长甩牌长度",
    ];
    let Some(stats) = stats.filter(|stats| stats.completed_games > 0) else {
        return LABELS
            .into_iter()
            .map(|label| (label, "--".to_owned()))
            .collect();
    };
    let games = f64::from(stats.completed_games);
    let mut rows = vec![
        ("对局数", stats.completed_games.to_string()),
        (
            "分数增减",
            format!("{:+.1}", stats.total_reference_delta as f64 / games),
        ),
        (
            "庄场得分",
            average_or_placeholder(stats.dealer_team_score, stats.dealer_team_games),
        ),
        (
            "闲场得分",
            average_or_placeholder(stats.collecting_team_score, stats.collecting_team_games),
        ),
        (
            "坐庄率",
            rate_or_placeholder(stats.dealer_games, stats.completed_games),
        ),
        (
            "亮主率",
            rate_or_placeholder(stats.declaration_games, stats.completed_games),
        ),
        (
            "反主率",
            rate_or_placeholder(stats.counter_games, stats.completed_games),
        ),
        (
            "保底率",
            rate_or_placeholder(stats.defended_kitty_games, stats.dealer_team_games),
        ),
        (
            "扣底率",
            rate_or_placeholder(stats.captured_kitty_games, stats.collecting_team_games),
        ),
        (
            "底牌平均分",
            average_or_placeholder(stats.buried_points, stats.buried_games),
        ),
        (
            "牌权率",
            rate_or_placeholder(stats.winning_plays, stats.plays),
        ),
        (
            "过江率",
            rate_or_placeholder(stats.crossing_games, stats.completed_games),
        ),
    ];
    rows.extend(
        LABELS[12..17]
            .iter()
            .copied()
            .zip(stats.play_category_counts)
            .map(|(label, count)| (label, count.to_string())),
    );
    rows.extend([
        ("最长拖拉机长度", stats.longest_tractor.to_string()),
        ("最长泰坦尼克长度", stats.longest_titanic.to_string()),
        ("最长太空堡垒长度", stats.longest_space_fortress.to_string()),
        ("最长甩牌长度", stats.longest_throw.to_string()),
    ]);
    rows
}

pub fn uno_profile_rows(stats: Option<&UnoProfileStats>) -> Vec<(&'static str, String)> {
    const LABELS: [&str; 21] = [
        "对局数",
        "分数增减",
        "场剩余分数",
        "平均顺位",
        "一位率",
        "二位率",
        "三位率",
        "四位率",
        "五位率",
        "六位率",
        "最多牌数",
        "最多被罚牌数",
        "最多被禁轮数",
        "UNO次数",
        "被罚UNO次数",
        "质疑次数",
        "质疑成功率",
        "被质疑次数",
        "被质疑成功率",
        "抢出次数",
        "抢出成功率",
    ];
    let Some(stats) = stats.filter(|stats| stats.completed_games > 0) else {
        return LABELS
            .into_iter()
            .map(|label| (label, "--".to_owned()))
            .collect();
    };
    let games = f64::from(stats.completed_games);
    let placement_total = stats
        .placement_counts
        .iter()
        .enumerate()
        .map(|(index, count)| (index + 1) as u64 * u64::from(*count))
        .sum::<u64>();
    let mut rows = vec![
        ("对局数", stats.completed_games.to_string()),
        (
            "分数增减",
            format!("{:+.1}", stats.total_reference_delta as f64 / games),
        ),
        (
            "场剩余分数",
            format!("{:.1}", stats.total_remaining_score as f64 / games),
        ),
        ("平均顺位", format!("{:.2}", placement_total as f64 / games)),
    ];
    for (label, count) in LABELS[4..10].iter().copied().zip(stats.placement_counts) {
        rows.push((label, format!("{:.1}%", f64::from(count) * 100.0 / games)));
    }
    rows.extend([
        ("最多牌数", stats.max_hand_cards.to_string()),
        ("最多被罚牌数", stats.max_penalty_cards.to_string()),
        ("最多被禁轮数", stats.max_skipped_turns.to_string()),
        ("UNO次数", stats.uno_calls.to_string()),
        ("被罚UNO次数", stats.uno_penalties.to_string()),
        ("质疑次数", stats.challenges.to_string()),
        (
            "质疑成功率",
            rate_or_placeholder(stats.successful_challenges, stats.challenges),
        ),
        ("被质疑次数", stats.challenges_received.to_string()),
        (
            "被质疑成功率",
            rate_or_placeholder(
                stats.successful_challenges_received,
                stats.challenges_received,
            ),
        ),
        ("抢出次数", stats.successful_jump_ins.to_string()),
        (
            "抢出成功率",
            rate_or_placeholder(stats.successful_jump_ins, stats.jump_in_opportunities),
        ),
    ]);
    rows
}
