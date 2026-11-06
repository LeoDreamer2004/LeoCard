use leocard_game_common::ActionTimeStats;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct MahjongLinkProfileStats {
    pub action_time: ActionTimeStats,
}
