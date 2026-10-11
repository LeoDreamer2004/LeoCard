use crate::AchievementDefinition;
use leocard_protocol::MatchId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A source-assigned event sequence, strictly increasing within one match.
/// Multiple facts in a single revision need distinct sequences. Snapshots must
/// never be assigned a context or submitted as live triggers.
#[derive(Clone, Copy, Debug)]
pub struct AchievementContext {
    pub match_id: MatchId,
    pub hand_index: Option<u32>,
    pub sequence: u128,
}

#[derive(Default, Deserialize, Serialize)]
pub(crate) struct AchievementProgress {
    pub(super) criteria: BTreeMap<String, CriterionProgress>,
    pub(super) earned_at: Option<u64>,
}

#[derive(Default, Deserialize, Serialize)]
pub(crate) struct CriterionProgress {
    pub(super) count: u64,
    pub(super) reached_at: Option<u64>,
    pub(super) scope: Option<([u8; 16], Option<u32>)>,
}

/// Pure local progress. The caller owns timestamps, persistence and delivery.
#[derive(Default, Deserialize, Serialize)]
pub struct AchievementBook {
    pub(super) progress: BTreeMap<String, AchievementProgress>,
    pub(super) receipts: BTreeMap<[u8; 16], u128>,
}

#[derive(Default)]
pub struct AchievementTriggerResult {
    pub progressed: bool,
    pub unlocked: Vec<&'static AchievementDefinition>,
}
