use super::GameProfileRows;
use super::rows::{
    ProfileRow, ProfileRowsBuilder, average_reference_delta, rate_or_placeholder,
    seconds_or_placeholder,
};
use leocard_game_common::{ActionTimeProfile, GameCountStats, PlacementStats, ReferenceStats};
use leocard_mahjong::{MAHJONG_MAJOR_FANS, MahjongProfileStats};

impl GameProfileRows for MahjongProfileStats {
    fn rows(stats: Option<&Self>) -> Vec<ProfileRow> {
        const LABELS: [&str; 14] = [
            "对局数",
            "分数增减",
            "场得分",
            "平均番数",
            "平均顺位",
            "一位率",
            "二位率",
            "三位率",
            "四位率",
            "和牌率",
            "点炮率",
            "自摸率",
            "流局率",
            "错和率",
        ];
        let mut labels = LABELS.to_vec();
        labels.extend(MAHJONG_MAJOR_FANS.iter().map(|fan| fan.name()));
        let action_time = (
            "平均每手用时",
            seconds_or_placeholder(
                stats.and_then(|stats| stats.action_time_stats().average_seconds()),
            ),
        );
        let Some(stats) = stats.filter(|stats| stats.completed_games() > 0) else {
            return ProfileRowsBuilder::unavailable(&labels, [action_time]);
        };
        let games = f64::from(stats.completed_games());
        let hands = stats.hands_played;
        let average_placement = stats
            .placement_counts()
            .iter()
            .enumerate()
            .map(|(index, count)| (index + 1) as u64 * u64::from(*count))
            .sum::<u64>() as f64
            / games;
        let mut rows = vec![
            ("对局数", stats.completed_games().to_string()),
            (
                "分数增减",
                average_reference_delta(stats.total_reference_delta(), stats.completed_games()),
            ),
            (
                "场得分",
                format!("{:.1}", stats.total_match_score as f64 / games),
            ),
            (
                "平均番数",
                if stats.wins == 0 {
                    "--".to_owned()
                } else {
                    format!("{:.1}", stats.total_win_fan as f64 / f64::from(stats.wins))
                },
            ),
            ("平均顺位", format!("{average_placement:.2}")),
        ];
        rows.extend(
            ["一位率", "二位率", "三位率", "四位率"]
                .into_iter()
                .zip(stats.placement_counts().iter().copied())
                .map(|(label, count)| (label, format!("{:.1}%", f64::from(count) * 100.0 / games))),
        );
        rows.extend(
            [
                ("和牌率", stats.wins),
                ("点炮率", stats.discards_into_win),
                ("自摸率", stats.self_draws),
                ("流局率", stats.exhaustive_draws),
                ("错和率", stats.false_wins),
            ]
            .into_iter()
            .map(|(label, count)| (label, rate_or_placeholder(count, hands))),
        );
        rows.extend(
            MAHJONG_MAJOR_FANS
                .iter()
                .zip(stats.major_fan_counts)
                .map(|(fan, count)| (fan.name(), count.to_string())),
        );
        rows.push(action_time);
        rows
    }
}
