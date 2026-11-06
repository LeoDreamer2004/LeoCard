use leocard_qigui523::{QiGui523ProfileStats, QiGuiPlayerStatistics};

pub(super) fn profile_statistics(progress: &QiGuiPlayerStatistics) -> QiGui523ProfileStats {
    QiGui523ProfileStats {
        straight_plays: progress.straight_plays,
        consecutive_pair_plays: progress.consecutive_pair_plays,
        airplane_plays: progress.airplane_plays,
        bomb_plays: progress.bomb_plays,
        heaven_bomb_plays: progress.heaven_bomb_plays,
        longest_straight: progress.longest_straight,
        longest_consecutive_pairs: progress.longest_consecutive_pairs,
        longest_airplane: progress.longest_airplane,
        ..Default::default()
    }
}
