use leocard_game_common::{ActionTimeProfile, ActionTimeStats};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
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
