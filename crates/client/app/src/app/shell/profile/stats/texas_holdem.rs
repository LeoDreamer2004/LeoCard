use super::rows::{ProfileRowsBuilder, average_or_placeholder, rate_or_placeholder};
use leocard_protocol::TexasHoldemProfileStats;

pub(crate) fn texas_holdem_profile_rows(
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
        return ProfileRowsBuilder::unavailable(&LABELS);
    };
    let mut rows = ProfileRowsBuilder::ranked(
        stats.completed_games,
        stats.total_reference_delta,
        "场筹码",
        stats.total_final_chips as f64,
        &stats.placement_counts,
        &LABELS[4..10],
    );
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
    rows.finish()
}
