use super::rows::{ProfileRowsBuilder, rate_or_placeholder};
use leocard_protocol::MinecraftGuessProfileStats;

pub(crate) fn minecraft_guess_profile_rows(
    stats: Option<&MinecraftGuessProfileStats>,
) -> Vec<(&'static str, String)> {
    const LABELS: [&str; 19] = [
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
        "七位率",
        "八位率",
        "总猜出次数",
        "猜出率",
        "方块猜出率",
        "物品猜出率",
        "跳过率",
        "错猜率",
        "猜测更优率",
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
        &LABELS[4..12],
    );
    rows.extend([
        ("总猜出次数", stats.wins().to_string()),
        (
            "猜出率",
            rate_or_placeholder(stats.wins(), stats.small_games()),
        ),
        (
            "方块猜出率",
            rate_or_placeholder(stats.block_wins, stats.block_games),
        ),
        (
            "物品猜出率",
            rate_or_placeholder(stats.item_wins, stats.item_games),
        ),
        ("跳过率", rate_or_placeholder(stats.skips, stats.actions())),
        (
            "错猜率",
            rate_or_placeholder(stats.unknown_guesses, stats.unknown_enabled_guesses),
        ),
        (
            "猜测更优率",
            rate_or_placeholder(stats.non_worse_guesses, stats.known_guesses),
        ),
    ]);
    rows.finish()
}
