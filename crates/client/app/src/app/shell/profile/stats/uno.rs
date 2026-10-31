use super::rows::{ProfileRowsBuilder, rate_or_placeholder};
use leocard_uno::UnoProfileStats;

pub(crate) fn uno_profile_rows(stats: Option<&UnoProfileStats>) -> Vec<(&'static str, String)> {
    const LABELS: [&str; 23] = [
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
        "七位率",
        "八位率",
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
        return ProfileRowsBuilder::unavailable(&LABELS);
    };
    let mut rows = ProfileRowsBuilder::ranked(
        stats.completed_games,
        stats.total_reference_delta,
        "场剩余分数",
        stats.total_remaining_score as f64,
        &stats.placement_counts,
        &LABELS[4..12],
    );
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
    rows.finish()
}
