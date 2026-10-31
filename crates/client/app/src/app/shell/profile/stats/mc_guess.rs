use super::rows::{ProfileRowsBuilder, rate_or_placeholder};
use leocard_mc_guess::{McGuessEntryKind, McGuessProfileStats};

pub(crate) fn mc_guess_profile_rows(
    stats: Option<&McGuessProfileStats>,
) -> Vec<(&'static str, String)> {
    const LABELS: [&str; 25] = [
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
        "生物猜出率",
        "附魔猜出率",
        "状态效果猜出率",
        "村民职业猜出率",
        "世界景观猜出率",
        "生物群系猜出率",
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
    ]);
    rows.extend(
        LABELS[14..22]
            .iter()
            .copied()
            .zip([
                McGuessEntryKind::Block,
                McGuessEntryKind::Item,
                McGuessEntryKind::Mob,
                McGuessEntryKind::Enchantment,
                McGuessEntryKind::Effect,
                McGuessEntryKind::VillagerProfession,
                McGuessEntryKind::Landscape,
                McGuessEntryKind::Biome,
            ])
            .map(|(label, kind)| {
                let counts = stats.categories.get(kind);
                (label, rate_or_placeholder(counts.wins, counts.games))
            }),
    );
    rows.extend([
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
