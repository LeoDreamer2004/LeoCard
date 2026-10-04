use leocard_protocol::{MatchId, PlayerReferenceChange, ShengjiProfileStats};
use leocard_shengji::ShengjiMatchStatistics;

#[derive(Clone, Debug, Default)]
pub(in crate::shengji) struct SessionStatistics {
    pub(in crate::shengji) hand: HandStatistics,
    pub(in crate::shengji) match_statistics: ShengjiMatchStatistics,
}

#[derive(Clone, Debug, Default)]
pub(in crate::shengji) struct HandStatistics {
    pub(in crate::shengji) profiles: Vec<ShengjiProfileStats>,
    pub(in crate::shengji) finished_settlement_id: Option<MatchId>,
    pub(in crate::shengji) finished_reference_changes: Option<Vec<PlayerReferenceChange>>,
}
