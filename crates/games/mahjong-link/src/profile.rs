use leocard_game_common::{ActionTimeProfile, ActionTimeStats};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct MahjongLinkProfileStats {
    pub action_time: ActionTimeStats,
}

impl ActionTimeProfile for MahjongLinkProfileStats {
    fn action_time_stats(&self) -> &ActionTimeStats {
        &self.action_time
    }

    fn action_time_stats_mut(&mut self) -> &mut ActionTimeStats {
        &mut self.action_time
    }
}
