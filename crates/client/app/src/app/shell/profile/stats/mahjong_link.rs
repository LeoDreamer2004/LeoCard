use super::rows::{ProfileRow, seconds_or_placeholder};
use leocard_mahjong_link::MahjongLinkProfileStats;

pub(crate) fn mahjong_link_profile_rows(
    stats: Option<&MahjongLinkProfileStats>,
) -> Vec<ProfileRow> {
    vec![(
        "平均每手用时",
        seconds_or_placeholder(stats.and_then(|stats| stats.action_time.average_seconds())),
    )]
}
