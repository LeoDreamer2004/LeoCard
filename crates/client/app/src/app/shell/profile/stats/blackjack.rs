use super::rows::{ProfileRowsBuilder, rate_or_placeholder};
use leocard_protocol::BlackjackProfileStats;

pub(crate) fn blackjack_profile_rows(
    stats: Option<&BlackjackProfileStats>,
) -> Vec<(&'static str, String)> {
    const LABELS: [&str; 18] = [
        "对局数",
        "分数增减",
        "场均筹码增减",
        "平均顺位",
        "一位率",
        "二位率",
        "三位率",
        "四位率",
        "五位率",
        "六位率",
        "参与小局",
        "手牌数",
        "胜率",
        "黑杰克",
        "爆牌率",
        "加倍次数",
        "分牌次数",
        "获胜手牌",
    ];
    let Some(stats) = stats.filter(|stats| stats.completed_games > 0) else {
        return ProfileRowsBuilder::unavailable(&LABELS);
    };
    let mut rows = ProfileRowsBuilder::ranked(
        stats.completed_games,
        stats.total_reference_delta,
        "场均筹码增减",
        stats.total_chips_delta as f64,
        &stats.placement_counts,
        &LABELS[4..10],
    );
    let counts = stats.statistics;
    rows.extend([
        ("参与小局", counts.rounds.to_string()),
        ("手牌数", counts.hands.to_string()),
        ("胜率", rate_or_placeholder(counts.wins, counts.hands)),
        ("黑杰克", counts.blackjacks.to_string()),
        ("爆牌率", rate_or_placeholder(counts.busts, counts.hands)),
        ("加倍次数", counts.doubles.to_string()),
        ("分牌次数", counts.splits.to_string()),
        ("获胜手牌", counts.wins.to_string()),
    ]);
    rows.finish()
}
