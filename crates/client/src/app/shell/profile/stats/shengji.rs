use super::rows::{ProfileRowsBuilder, average_or_placeholder, rate_or_placeholder};
use leocard_protocol::ShengjiProfileStats;

pub(crate) fn shengji_profile_rows(
    stats: Option<&ShengjiProfileStats>,
) -> Vec<(&'static str, String)> {
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
        "宇宙飞船次数",
        "甩牌次数",
        "最长拖拉机长度",
        "最长泰坦尼克长度",
        "最长宇宙飞船长度",
        "最长甩牌长度",
    ];
    let Some(stats) = stats.filter(|stats| stats.completed_games > 0) else {
        return ProfileRowsBuilder::unavailable(&LABELS);
    };
    let mut rows = ProfileRowsBuilder::plain(stats.completed_games, stats.total_reference_delta);
    rows.extend([
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
    ]);
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
        ("最长宇宙飞船长度", stats.longest_spaceship.to_string()),
        ("最长甩牌长度", stats.longest_throw.to_string()),
    ]);
    rows.finish()
}
