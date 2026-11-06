use super::GameProfileRows;
use super::rows::{ProfileRow, seconds_or_placeholder};
use leocard_game_common::ActionTimeProfile;
use leocard_mahjong_link::MahjongLinkProfileStats;

impl GameProfileRows for MahjongLinkProfileStats {
    fn rows(stats: Option<&Self>) -> Vec<ProfileRow> {
        vec![(
            "平均每手用时",
            seconds_or_placeholder(
                stats.and_then(|stats| stats.action_time_stats().average_seconds()),
            ),
        )]
    }
}
