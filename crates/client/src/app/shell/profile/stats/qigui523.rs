use super::rows::ProfileRowsBuilder;
use leocard_protocol::QiGui523ProfileStats;

pub(crate) fn qigui523_profile_rows(
    stats: Option<&QiGui523ProfileStats>,
) -> Vec<(&'static str, String)> {
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
        return ProfileRowsBuilder::unavailable(&LABELS);
    };
    let mut rows = ProfileRowsBuilder::ranked(
        stats.completed_games,
        stats.total_reference_delta,
        "场得分",
        stats.total_score as f64,
        &stats.placement_counts,
        &LABELS[4..10],
    );
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
    rows.finish()
}
