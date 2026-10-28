use super::state::{HandStatistics, SessionStatistics};
use leocard_protocol::ShengjiProfileStats;

impl SessionStatistics {
    pub(in crate::shengji) fn start_hand(&mut self, player_count: u8) {
        self.hand = HandStatistics {
            profiles: vec![ShengjiProfileStats::default(); usize::from(player_count)],
            ..HandStatistics::default()
        };
    }
}
