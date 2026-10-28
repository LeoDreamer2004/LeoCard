use super::rows::{ProfileRowsBuilder, average_or_placeholder, rate_or_placeholder};
use leocard_protocol::DoudizhuProfileStats;

pub(crate) fn doudizhu_profile_rows(
    stats: Option<&DoudizhuProfileStats>,
) -> Vec<(&'static str, String)> {
    const LABELS: [&str; 12] = [
        "对局数",
        "分数增减",
        "场得分",
        "地主率",
        "平均叫分",
        "顺子次数",
        "连对次数",
        "飞机次数",
        "炸弹次数",
        "顺子最长长度",
        "连对最长长度",
        "飞机最长长度",
    ];
    let Some(stats) = stats.filter(|stats| stats.completed_games > 0) else {
        return ProfileRowsBuilder::unavailable(&LABELS);
    };
    let mut rows = ProfileRowsBuilder::plain(stats.completed_games, stats.total_reference_delta);
    rows.extend([
        (
            "场得分",
            format!(
                "{:.1}",
                stats.total_match_score as f64 / f64::from(stats.completed_games)
            ),
        ),
        (
            "地主率",
            rate_or_placeholder(stats.landlord_hands, stats.hands_played),
        ),
        (
            "平均叫分",
            average_or_placeholder(stats.total_bid_score, stats.bid_actions),
        ),
        ("顺子次数", stats.straight_plays.to_string()),
        ("连对次数", stats.consecutive_pair_plays.to_string()),
        ("飞机次数", stats.airplane_plays.to_string()),
        ("炸弹次数", stats.bomb_plays.to_string()),
        ("顺子最长长度", stats.longest_straight.to_string()),
        ("连对最长长度", stats.longest_consecutive_pairs.to_string()),
        ("飞机最长长度", stats.longest_airplane.to_string()),
    ]);
    rows.finish()
}
