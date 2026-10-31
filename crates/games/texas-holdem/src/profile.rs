//! Long-term statistics owned by this game. Field order is part of the saved profile format.

use crate::TexasRuleSet;

/// 普通德州扑克与奥马哈共用的长期档案。
#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct TexasProfileStats {
    pub completed_games: u32,
    pub total_reference_delta: i64,
    pub total_final_chips: u64,
    pub placement_counts: [u32; TexasRuleSet::MAX_PLAYERS as usize],
    /// 跟注、加注和全下实际投入的筹码总和；不含盲注。
    pub wagered_chips: u64,
    pub wager_actions: u32,
    /// 不含盲注的弃牌、过牌、跟注、加注及全下动作总数。
    pub voluntary_actions: u32,
    pub check_actions: u32,
    pub raise_actions: u32,
    pub all_in_actions: u32,
    pub hands_played: u32,
    pub hands_folded: u32,
    /// 顺序依次为高牌、一对、两对、三条、顺子、同花、葫芦、四条、
    /// 同花顺及皇家同花顺。
    pub hand_category_counts: [u32; 10],
    /// 已采集入池情况的小局数，不含无法还原该信息的旧档案。
    pub entry_tracked_hands: u32,
    /// 进入翻牌或翻前直接获胜的小局数。
    pub entered_hands: u32,
}
