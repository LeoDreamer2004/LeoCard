use leocard_protocol::ActionTimeStats;

pub(super) type ProfileRow = (&'static str, String);

pub(crate) fn append_action_time_row(rows: &mut Vec<ProfileRow>, stats: ActionTimeStats) {
    let value = if stats.completed_actions == 0 {
        "暂无记录".to_owned()
    } else {
        format!(
            "{:.1} 秒",
            stats.total_millis as f64 / stats.completed_actions as f64 / 1000.0
        )
    };
    rows.push(("平均每手用时", value));
}

pub(super) fn average_reference_delta(total: i64, completed_games: u32) -> String {
    format!("{:+.3}", total as f64 / f64::from(completed_games))
}

pub(super) struct ProfileRowsBuilder {
    rows: Vec<ProfileRow>,
}

impl ProfileRowsBuilder {
    pub(super) fn unavailable(labels: &[&'static str]) -> Vec<ProfileRow> {
        labels
            .iter()
            .map(|label| (*label, "--".to_owned()))
            .collect()
    }

    pub(super) fn ranked(
        completed_games: u32,
        total_reference_delta: i64,
        average_label: &'static str,
        average_total: f64,
        placement_counts: &[u32],
        placement_labels: &[&'static str],
    ) -> Self {
        let games = f64::from(completed_games);
        let placement_total = placement_counts
            .iter()
            .enumerate()
            .map(|(index, count)| (index + 1) as u64 * u64::from(*count))
            .sum::<u64>();
        let mut rows = vec![
            ("对局数", completed_games.to_string()),
            (
                "分数增减",
                average_reference_delta(total_reference_delta, completed_games),
            ),
            (average_label, format!("{:.1}", average_total / games)),
            ("平均顺位", format!("{:.2}", placement_total as f64 / games)),
        ];
        rows.extend(
            placement_labels
                .iter()
                .copied()
                .zip(placement_counts)
                .map(|(label, count)| {
                    (label, format!("{:.1}%", f64::from(*count) * 100.0 / games))
                }),
        );
        Self { rows }
    }

    pub(super) fn plain(completed_games: u32, total_reference_delta: i64) -> Self {
        Self {
            rows: vec![
                ("对局数", completed_games.to_string()),
                (
                    "分数增减",
                    average_reference_delta(total_reference_delta, completed_games),
                ),
            ],
        }
    }

    pub(super) fn extend(&mut self, rows: impl IntoIterator<Item = ProfileRow>) {
        self.rows.extend(rows);
    }

    pub(super) fn finish(self) -> Vec<ProfileRow> {
        self.rows
    }
}

pub(super) fn average_or_placeholder(total: u64, count: u32) -> String {
    if count == 0 {
        "--".to_owned()
    } else {
        format!("{:.1}", total as f64 / f64::from(count))
    }
}

pub(super) fn rate_or_placeholder(count: u32, total: u32) -> String {
    if total == 0 {
        "--".to_owned()
    } else {
        format!(
            "{:.1}%",
            f64::from(count.min(total)) * 100.0 / f64::from(total)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_time_row_uses_weighted_seconds_and_distinguishes_missing_data() {
        for (total_millis, completed_actions, expected) in [
            (0, 0, "暂无记录"),
            (0, 5, "0.0 秒"),
            (23_000, 10, "2.3 秒"),
            (10_000, 3, "3.3 秒"),
        ] {
            let mut rows = vec![("对局数", "100".to_owned())];
            append_action_time_row(
                &mut rows,
                ActionTimeStats {
                    total_millis,
                    completed_actions,
                    ..Default::default()
                },
            );
            assert_eq!(rows[0], ("对局数", "100".to_owned()));
            assert_eq!(rows[1], ("平均每手用时", expected.to_owned()));
        }
    }
}
