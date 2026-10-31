use super::TexasAdapter;
use crate::automation::BotDecision;
use crate::{AutomaticActionDelayState, RoomSession};
use leocard_protocol::PlayerReferenceChange;
use leocard_texas_holdem::{
    TexasAction, TexasBotObservation, TexasCard, TexasProfileStats, TexasRuleSet,
};

/// 复用 [`RoomSession`] 的德州扑克权威房间后端。
#[derive(Clone, Debug)]
pub struct TexasSession {
    pub(super) bot_decision: BotDecision<TexasBotObservation, TexasAction>,
    pub(super) room: RoomSession,
    pub(super) rules: TexasRuleSet,
    pub(super) shuffled_deck: Option<Vec<TexasCard>>,
    pub(super) game: Option<TexasAdapter>,
    pub(super) match_profile_stats: Vec<TexasProfileStats>,
    pub(super) finished_reference_changes: Option<Vec<PlayerReferenceChange>>,
    pub(super) automatic_action_delay: Option<AutomaticActionDelayState>,
}
