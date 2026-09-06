use crate::{AutoPlayDelayState, RoomSession, TurnTimerState};
use leocard_protocol::{MatchId, PlayerReferenceChange, QiGui523ProfileStats};
use leocard_qigui523::{GameState, QiGuiCard, QiGuiRuleSet};
use std::ops::{Deref, DerefMut};

/// 单房间权威会话。所有命令均按调用顺序串行处理。
#[derive(Clone, Debug)]
pub struct QiGui523Session {
    pub(super) room: RoomSession,
    pub(super) rules: QiGuiRuleSet,
    pub(super) shuffled_deck: Option<Vec<QiGuiCard>>,
    pub(super) game: Option<GameState>,
    pub(super) match_id: Option<MatchId>,
    pub(super) finished_reference_changes: Option<Vec<PlayerReferenceChange>>,
    pub(super) match_profile_stats: Vec<QiGui523ProfileStats>,
    pub(super) turn_timer: Option<TurnTimerState>,
    pub(super) auto_play_delay: Option<AutoPlayDelayState>,
}

impl Deref for QiGui523Session {
    type Target = RoomSession;

    fn deref(&self) -> &Self::Target {
        &self.room
    }
}

impl DerefMut for QiGui523Session {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.room
    }
}
