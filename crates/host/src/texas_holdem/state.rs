use super::TexasHoldemAdapter;
use crate::{AutoPlayDelayState, RoomSession};
use leocard_protocol::{PlayerReferenceChange, TexasHoldemProfileStats};
use leocard_texas_holdem::{TexasHoldemCard, TexasHoldemRuleSet};

/// 复用 [`RoomSession`] 的德州扑克权威房间后端。
#[derive(Clone, Debug)]
pub struct TexasHoldemSession {
    pub(super) room: RoomSession,
    pub(super) rules: TexasHoldemRuleSet,
    pub(super) shuffled_deck: Option<Vec<TexasHoldemCard>>,
    pub(super) game: Option<TexasHoldemAdapter>,
    pub(super) match_profile_stats: Vec<TexasHoldemProfileStats>,
    pub(super) finished_reference_changes: Option<Vec<PlayerReferenceChange>>,
    pub(super) auto_play_delay: Option<AutoPlayDelayState>,
}
