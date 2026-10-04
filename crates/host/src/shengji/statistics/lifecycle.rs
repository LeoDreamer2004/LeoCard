use super::state::{HandStatistics, SessionStatistics};
use leocard_protocol::ShengjiProfileStats;
use leocard_shengji::ShengjiRuleSet;

impl SessionStatistics {
    pub(in crate::shengji) fn start_hand(&mut self) {
        self.hand = HandStatistics {
            profiles: vec![ShengjiProfileStats::default(); ShengjiRuleSet::PLAYER_COUNT],
            ..HandStatistics::default()
        };
    }
}
