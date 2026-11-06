use super::rows::{ProfileRowsBuilder, rate_or_placeholder, seconds_or_placeholder};
use leocard_guandan::GuandanProfileStats;

pub(crate) fn guandan_profile_rows(
    stats: Option<&GuandanProfileStats>,
) -> Vec<(&'static str, String)> {
    const LABELS: [&str; 18] = [
        "对局数",
        "分数增减",
        "整场胜率",
        "参与小局",
        "小局胜率",
        "头游率",
        "二游率",
        "三游率",
        "末游率",
        "未定游次率",
        "双上率",
        "累计升级",
        "顺子次数",
        "三连对次数",
        "钢板次数",
        "普通炸弹次数",
        "同花顺次数",
        "四王炸次数",
    ];
    let action_time = (
        "平均每手用时",
        seconds_or_placeholder(stats.and_then(|stats| stats.action_time.average_seconds())),
    );
    let Some(stats) = stats.filter(|stats| stats.completed_games > 0) else {
        return ProfileRowsBuilder::unavailable(&LABELS, [action_time]);
    };
    let mut rows = ProfileRowsBuilder::plain(stats.completed_games, stats.total_reference_delta);
    rows.extend([
        (
            "整场胜率",
            rate_or_placeholder(stats.match_wins, stats.completed_games),
        ),
        ("参与小局", stats.hands_played.to_string()),
        (
            "小局胜率",
            rate_or_placeholder(stats.hands_won, stats.hands_played),
        ),
    ]);
    rows.extend(
        LABELS[5..9]
            .iter()
            .copied()
            .zip(stats.placement_counts)
            .map(|(label, count)| (label, rate_or_placeholder(count, stats.hands_played))),
    );
    rows.extend([
        (
            "未定游次率",
            rate_or_placeholder(stats.unranked_hands, stats.hands_played),
        ),
        (
            "双上率",
            rate_or_placeholder(stats.double_finish_hands, stats.hands_played),
        ),
        ("累计升级", stats.levels_advanced.to_string()),
        ("顺子次数", stats.straight_plays.to_string()),
        ("三连对次数", stats.consecutive_pair_plays.to_string()),
        ("钢板次数", stats.consecutive_triple_plays.to_string()),
        ("普通炸弹次数", stats.bomb_plays.to_string()),
        ("同花顺次数", stats.straight_flush_plays.to_string()),
        ("四王炸次数", stats.joker_bomb_plays.to_string()),
    ]);
    rows.extend([action_time]);
    rows.finish()
}
